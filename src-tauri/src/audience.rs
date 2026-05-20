#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstrumentAudience {
    pub public: String,
    pub column: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstrumentColumn {
    pub key: String,
    pub label: String,
}

impl InstrumentAudience {
    pub fn fallback() -> Self {
        Self {
            public: "General".into(),
            column: "General".into(),
        }
    }

    pub fn parse(value: &str) -> Self {
        let normalized = normalize_audience_label(value);
        for prefix in known_public_prefixes() {
            if normalized.eq_ignore_ascii_case(prefix) {
                return Self {
                    public: (*prefix).into(),
                    column: (*prefix).into(),
                };
            }
            let boundary = format!("{prefix} ");
            if normalized
                .to_lowercase()
                .starts_with(&boundary.to_lowercase())
            {
                return Self {
                    public: (*prefix).into(),
                    column: normalized,
                };
            }
        }

        Self {
            public: "General".into(),
            column: normalized,
        }
    }

    pub fn subpublic(&self) -> String {
        if self.public == "General" {
            return self.column.clone();
        }
        self.column
            .strip_prefix(&format!("{} ", self.public))
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(&self.column)
            .to_string()
    }
}

pub fn audience_from_excel(public: String, subpublic: String) -> String {
    let public = normalize_audience_label(public);
    let subpublic = normalize_audience_label(subpublic);
    match (public.is_empty(), subpublic.is_empty()) {
        (true, true) => String::new(),
        (false, true) => public,
        (true, false) => subpublic,
        (false, false) => format!("{public} {subpublic}"),
    }
}

pub fn normalize_audience_label(value: impl AsRef<str>) -> String {
    value
        .as_ref()
        .trim()
        .replace('_', " ")
        .split_whitespace()
        .map(|part| {
            part.trim_start_matches(|character: char| {
                character.is_ascii_digit() || character.is_whitespace()
            })
        })
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn display_public_label(public: &str) -> String {
    match public {
        "Profesores Planta" => "Profesores de planta".into(),
        "Profesores Cátedra" | "Profesores Catedra" => "Profesores de cátedra".into(),
        _ => public.into(),
    }
}

pub fn provider_instrument_key(value: &str) -> String {
    let parsed = InstrumentAudience::parse(value);
    if parsed.public == "General" {
        parsed.column
    } else {
        parsed.public
    }
}

pub fn provider_instrument_label(key: &str) -> String {
    if key == "Sin instrumento" {
        key.into()
    } else {
        display_public_label(key)
    }
}

pub fn instrument_title_public(public: &str) -> String {
    match public {
        "Profesores Planta" => "PROFESORES DE PLANTA".into(),
        "Profesores Cátedra" | "Profesores Catedra" => "PROFESORES DE CÁTEDRA".into(),
        _ => public.to_uppercase(),
    }
}

pub fn display_instrument_column_label(public: &str, key: &str) -> String {
    let audience = InstrumentAudience::parse(key);
    let subpublic = canonical_subpublic_label(&audience.subpublic());
    if public == "General" {
        return subpublic;
    }
    let prefix = instrument_column_prefix(public);
    if subpublic.is_empty() || subpublic == prefix {
        prefix.into()
    } else {
        format!("{prefix} {subpublic}")
    }
}

fn instrument_column_prefix(public: &str) -> &str {
    match public {
        "Profesores Planta" | "Profesores Cátedra" | "Profesores Catedra" => "Profesores",
        _ => public,
    }
}

fn canonical_subpublic_label(value: &str) -> String {
    match value {
        "Maestrías" | "Maestrias" => "Maestría".into(),
        "Maestrías virtuales" | "Maestrias virtuales" => "Maestría Virtual".into(),
        "Especializaciones MQ" => "EMQ".into(),
        "Especializaciones virtuales/extensión" | "Especializaciones virtuales/extension" => {
            "Especializaciones virtual / extensión".into()
        }
        _ => value.into(),
    }
}

fn known_public_prefixes() -> &'static [&'static str] {
    &[
        "Profesores Planta",
        "Profesores Cátedra",
        "Profesores Catedra",
        "Servicios Generales",
        "Administrativos",
        "Directivos",
        "Estudiantes",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_encoded_excel_publics_and_subpublics() {
        assert_eq!(normalize_audience_label("10Pregrado"), "Pregrado");
        assert_eq!(
            normalize_audience_label("1Profesores_Planta"),
            "Profesores Planta"
        );
        assert_eq!(
            audience_from_excel("0Estudiantes".into(), "02Maestrías virtuales".into()),
            "Estudiantes Maestrías virtuales"
        );
    }

    #[test]
    fn derives_instrument_group_and_display_columns() {
        let audience = InstrumentAudience::parse("1Profesores_Planta 10Pregrado");
        assert_eq!(audience.public, "Profesores Planta");
        assert_eq!(audience.column, "Profesores Planta Pregrado");
        assert_eq!(audience.subpublic(), "Pregrado");
        assert_eq!(
            display_instrument_column_label(&audience.public, &audience.column),
            "Profesores Pregrado"
        );
        assert_eq!(
            display_instrument_column_label("Estudiantes", "Estudiantes Maestrías virtuales"),
            "Estudiantes Maestría Virtual"
        );
    }
}
