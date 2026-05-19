use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use url::Url;
use uuid::Uuid;

use crate::domain::{MicrosoftAccount, MicrosoftLoginRequest, MicrosoftLoginResult};
use crate::error::AppError;

const SCOPES: [&str; 5] = [
    "openid",
    "profile",
    "email",
    "User.Read",
    "Files.ReadWrite.AppFolder",
];

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    scope: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphMe {
    display_name: Option<String>,
    mail: Option<String>,
    user_principal_name: Option<String>,
}

pub async fn login_with_microsoft(
    request: MicrosoftLoginRequest,
) -> Result<(MicrosoftLoginResult, String), AppError> {
    let tenant_id = normalize_tenant(&request.tenant_id);
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let redirect_uri = format!("http://localhost:{port}/auth/microsoft/callback");
    let state = Uuid::new_v4().to_string();
    let code_verifier = pkce_verifier();
    let code_challenge = pkce_challenge(&code_verifier);
    let scope = SCOPES.join(" ");

    let auth_url = Url::parse_with_params(
        &format!("https://login.microsoftonline.com/{tenant_id}/oauth2/v2.0/authorize"),
        &[
            ("client_id", request.client_id.as_str()),
            ("response_type", "code"),
            ("redirect_uri", redirect_uri.as_str()),
            ("response_mode", "query"),
            ("scope", scope.as_str()),
            ("state", state.as_str()),
            ("code_challenge", code_challenge.as_str()),
            ("code_challenge_method", "S256"),
            ("prompt", "select_account"),
        ],
    )?;

    open::that(auth_url.as_str()).map_err(|error| AppError::BrowserOpen(error.to_string()))?;
    let code = wait_for_callback(listener, &state).await?;
    let token = exchange_code(
        &tenant_id,
        &request.client_id,
        &redirect_uri,
        &code,
        &code_verifier,
    )
    .await?;
    let account = fetch_graph_profile(&token.access_token).await?;

    Ok((
        MicrosoftLoginResult {
            account,
            tenant_id,
            scopes: token
                .scope
                .unwrap_or(scope)
                .split_whitespace()
                .map(ToOwned::to_owned)
                .collect(),
        },
        token.access_token,
    ))
}

pub async fn upload_database_to_app_folder(
    access_token: &str,
    database_path: &std::path::Path,
) -> Result<(), AppError> {
    let bytes = std::fs::read(database_path)?;
    reqwest::Client::new()
        .put("https://graph.microsoft.com/v1.0/me/drive/special/approot:/autoevaluacion-cna.db:/content")
        .bearer_auth(access_token)
        .body(bytes)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

pub async fn download_database_from_app_folder(
    access_token: &str,
    database_path: &std::path::Path,
) -> Result<(), AppError> {
    let bytes = reqwest::Client::new()
        .get("https://graph.microsoft.com/v1.0/me/drive/special/approot:/autoevaluacion-cna.db:/content")
        .bearer_auth(access_token)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    if let Some(parent) = database_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(database_path, bytes)?;
    Ok(())
}

async fn wait_for_callback(
    listener: TcpListener,
    expected_state: &str,
) -> Result<String, AppError> {
    let (mut stream, _) = listener.accept().await?;
    let mut buffer = vec![0; 8192];
    let read = stream.read(&mut buffer).await?;
    let request = String::from_utf8_lossy(&buffer[..read]);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or_else(|| AppError::Validation("invalid microsoft callback request".into()))?;
    let callback = Url::parse(&format!("http://localhost{path}"))?;
    let params = callback.query_pairs().collect::<Vec<_>>();
    let state = params
        .iter()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string())
        .unwrap_or_default();
    let code = params
        .iter()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.to_string());
    let error = params
        .iter()
        .find(|(key, _)| key == "error_description" || key == "error")
        .map(|(_, value)| value.to_string());

    let response = "HTTP/1.1 200 OK\r\ncontent-type: text/html; charset=utf-8\r\n\r\n<html><body><h1>Autoevaluacion CNA</h1><p>Microsoft login completed. You can return to the app.</p></body></html>";
    stream.write_all(response.as_bytes()).await?;

    if state != expected_state {
        return Err(AppError::Validation(
            "microsoft login state did not match the local request".into(),
        ));
    }

    code.ok_or_else(|| {
        AppError::Validation(
            error.unwrap_or_else(|| "microsoft login did not return an authorization code".into()),
        )
    })
}

async fn exchange_code(
    tenant_id: &str,
    client_id: &str,
    redirect_uri: &str,
    code: &str,
    code_verifier: &str,
) -> Result<TokenResponse, AppError> {
    let endpoint = format!("https://login.microsoftonline.com/{tenant_id}/oauth2/v2.0/token");
    let response = reqwest::Client::new()
        .post(endpoint)
        .form(&[
            ("client_id", client_id),
            ("scope", &SCOPES.join(" ")),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
            ("code_verifier", code_verifier),
        ])
        .send()
        .await?
        .error_for_status()?
        .json::<TokenResponse>()
        .await?;
    Ok(response)
}

async fn fetch_graph_profile(access_token: &str) -> Result<MicrosoftAccount, AppError> {
    let me = reqwest::Client::new()
        .get("https://graph.microsoft.com/v1.0/me?$select=displayName,mail,userPrincipalName")
        .bearer_auth(access_token)
        .send()
        .await?
        .error_for_status()?
        .json::<GraphMe>()
        .await?;
    let email = me
        .mail
        .or(me.user_principal_name)
        .ok_or_else(|| AppError::Validation("microsoft profile did not include an email".into()))?;
    let display_name = me.display_name.clone().unwrap_or_else(|| email.clone());

    Ok(MicrosoftAccount {
        email,
        display_name,
    })
}

fn pkce_verifier() -> String {
    format!(
        "{}{}{}",
        Uuid::new_v4().simple(),
        Uuid::new_v4().simple(),
        Uuid::new_v4().simple()
    )
}

fn pkce_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

fn normalize_tenant(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "organizations".into()
    } else {
        trimmed.into()
    }
}
