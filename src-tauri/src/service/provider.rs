use std::path::Path;

use docx_rs::{Docx, Paragraph, Pic, Run, Shading, ShdType, Table, TableCell, TableRow, WidthType};

use crate::domain::{
    ExportProviderReviewDocxRequest, NewProviderLink, ProviderLink, ProviderQuestionReview,
    ProviderQuestionReviewItem, ResetProviderQuestionReviewsRequest,
    ResetProviderQuestionReviewsResult, SaveProviderQuestionReviewRequest,
};
use crate::error::AppError;
use crate::service::AutoEvaluationService;

impl AutoEvaluationService {
    pub async fn record_provider_link(
        &self,
        link: NewProviderLink,
    ) -> Result<ProviderLink, AppError> {
        if !link.url.starts_with("https://") {
            return Err(AppError::Validation(
                "provider links must use an https URL".into(),
            ));
        }

        self.repository.insert_provider_link(link).await
    }

    pub async fn list_provider_links(&self) -> Result<Vec<ProviderLink>, AppError> {
        self.repository.list_provider_links().await
    }

    pub async fn list_provider_question_review_items(
        &self,
    ) -> Result<Vec<ProviderQuestionReviewItem>, AppError> {
        let questions = self.repository.list_questions().await?;
        let reviews = self.repository.list_provider_question_reviews().await?;
        let mut by_question_and_instrument = std::collections::HashMap::new();
        let mut legacy_by_question = std::collections::HashMap::new();
        for review in reviews {
            if review.instrument_audience.trim().is_empty() {
                legacy_by_question.insert(review.question_id.clone(), review);
            } else {
                let instrument = normalize_instrument_audience(&review.instrument_audience);
                by_question_and_instrument.insert((review.question_id.clone(), instrument), review);
            }
        }

        let mut items = Vec::new();
        for question in questions {
            let audiences = if question.audiences.is_empty() {
                vec!["Sin instrumento".to_string()]
            } else {
                normalize_instrument_audiences(&question.audiences)
            };
            for audience in audiences {
                let review = by_question_and_instrument
                    .get(&(question.id.clone(), audience.clone()))
                    .cloned()
                    .or_else(|| legacy_by_question.get(&question.id).cloned());
                items.push(ProviderQuestionReviewItem {
                    question: question.clone(),
                    instrument_audience: audience,
                    review,
                });
            }
        }

        items.sort_by(|left, right| {
            left.instrument_audience
                .cmp(&right.instrument_audience)
                .then(left.question.code.cmp(&right.question.code))
        });
        Ok(items)
    }

    pub async fn save_provider_question_review(
        &self,
        mut review: SaveProviderQuestionReviewRequest,
    ) -> Result<ProviderQuestionReview, AppError> {
        review.instrument_audience = normalize_instrument_audience(&review.instrument_audience);
        if review.observation.trim().is_empty()
            && !matches!(
                review.status,
                crate::domain::ProviderQuestionReviewStatus::Correct
            )
        {
            return Err(AppError::Validation(
                "observation is required when a question is not correct".into(),
            ));
        }
        self.repository.save_provider_question_review(review).await
    }

    pub async fn reset_provider_question_reviews(
        &self,
        request: ResetProviderQuestionReviewsRequest,
    ) -> Result<ResetProviderQuestionReviewsResult, AppError> {
        if request.confirmation_text.trim() != "REINICIAR REVISION" {
            return Err(AppError::Validation(
                "confirmation text must be REINICIAR REVISION".into(),
            ));
        }

        let deleted_reviews = self.repository.reset_provider_question_reviews().await?;
        Ok(ResetProviderQuestionReviewsResult { deleted_reviews })
    }

    pub async fn export_provider_review_docx(
        &self,
        request: ExportProviderReviewDocxRequest,
    ) -> Result<(), AppError> {
        let selected_instrument = request
            .instrument_audience
            .as_deref()
            .map(normalize_instrument_audience)
            .filter(|value| !value.is_empty());
        let items = self
            .list_provider_question_review_items()
            .await?
            .into_iter()
            .filter(|item| {
                selected_instrument
                    .as_ref()
                    .map(|instrument| item.instrument_audience == *instrument)
                    .unwrap_or(true)
            })
            .collect::<Vec<_>>();
        if items.is_empty() {
            return Err(AppError::Validation(
                "no provider review items were found for the selected instrument".into(),
            ));
        }

        let instrument_label = selected_instrument
            .as_deref()
            .unwrap_or("Todos los instrumentos");
        let total = items.len();
        let approved = items
            .iter()
            .filter(|item| {
                item.review
                    .as_ref()
                    .map(|review| {
                        matches!(
                            review.status,
                            crate::domain::ProviderQuestionReviewStatus::Correct
                        )
                    })
                    .unwrap_or(false)
            })
            .count();
        let missing = items
            .iter()
            .filter(|item| {
                item.review
                    .as_ref()
                    .map(|review| {
                        matches!(
                            review.status,
                            crate::domain::ProviderQuestionReviewStatus::Missing
                        )
                    })
                    .unwrap_or(false)
            })
            .count();
        let needs_modification = items
            .iter()
            .filter(|item| {
                item.review
                    .as_ref()
                    .map(|review| {
                        matches!(
                            review.status,
                            crate::domain::ProviderQuestionReviewStatus::NeedsModification
                        )
                    })
                    .unwrap_or(false)
            })
            .count();
        let pending = total - approved - missing - needs_modification;

        let mut doc = Docx::new()
            .add_paragraph(
                Paragraph::new()
                    .add_run(
                        Run::new()
                            .add_text("Revision de preguntas del proveedor")
                            .bold()
                            .size(36)
                            .color("111827"),
                    )
                    .style("Title"),
            )
            .add_paragraph(
                Paragraph::new().add_run(
                    Run::new()
                        .add_text(format!("Instrumento: {instrument_label}"))
                        .bold()
                        .size(26)
                        .color("2563EB"),
                ),
            )
            .add_paragraph(Paragraph::new().add_run(
                Run::new()
                    .add_text("Reporte de verificacion por pregunta, estado, observacion y evidencia.")
                    .size(22)
                    .color("4B5563"),
            ))
            .add_paragraph(Paragraph::new())
            .add_table(summary_table(total, approved, needs_modification, missing, pending))
            .add_paragraph(Paragraph::new())
            .add_paragraph(
                Paragraph::new().add_run(
                    Run::new()
                        .add_text("Detalle de revision")
                        .bold()
                        .size(28)
                        .color("111827"),
                ),
            );

        for item in items {
            let status = item
                .review
                .as_ref()
                .map(|review| review_status_label(&review.status))
                .unwrap_or("Pendiente");
            let observation = item
                .review
                .as_ref()
                .map(|review| review.observation.as_str())
                .unwrap_or("");
            let evidence = item
                .review
                .as_ref()
                .and_then(|review| review.evidence_path.as_deref())
                .unwrap_or("");
            doc = doc
                .add_paragraph(Paragraph::new())
                .add_paragraph(
                    Paragraph::new().add_run(
                        Run::new()
                            .add_text(format!("{} · {}", item.question.code, status))
                            .bold()
                            .size(24)
                            .color(status_color(status)),
                    ),
                )
                .add_paragraph(
                    Paragraph::new().add_run(
                        Run::new()
                            .add_text(item.question.text)
                            .size(22)
                            .color("111827"),
                    ),
                )
                .add_paragraph(
                    Paragraph::new()
                        .add_run(Run::new().add_text("Observacion: ").bold().size(20))
                        .add_run(
                            Run::new()
                                .add_text(if observation.is_empty() {
                                    "Sin observacion"
                                } else {
                                    observation
                                })
                                .size(20)
                                .color("374151"),
                        ),
                )
                .add_paragraph(
                    Paragraph::new()
                        .add_run(Run::new().add_text("Evidencia: ").bold().size(20))
                        .add_run(
                            Run::new()
                                .add_text(if evidence.is_empty() {
                                    "Sin evidencia adjunta"
                                } else {
                                    evidence
                                })
                                .size(20)
                                .color("374151"),
                        ),
                );
            if let Some(evidence_paragraph) = evidence_image_paragraph(evidence) {
                doc = doc.add_paragraph(evidence_paragraph);
            }
        }
        let file = std::fs::File::create(request.path)?;
        doc.build()
            .pack(file)
            .map_err(|error| AppError::Validation(format!("word export failed: {error}")))?;
        Ok(())
    }
}

fn summary_table(
    total: usize,
    approved: usize,
    needs_modification: usize,
    missing: usize,
    pending: usize,
) -> Table {
    Table::new(vec![
        TableRow::new(vec![
            summary_header_cell("Total"),
            summary_header_cell("OK"),
            summary_header_cell("Modificar"),
            summary_header_cell("No aparece"),
            summary_header_cell("Pendiente"),
        ]),
        TableRow::new(vec![
            summary_value_cell(total),
            summary_value_cell(approved),
            summary_value_cell(needs_modification),
            summary_value_cell(missing),
            summary_value_cell(pending),
        ]),
    ])
    .width(5000, WidthType::Pct)
}

fn summary_header_cell(label: &str) -> TableCell {
    TableCell::new()
        .width(1000, WidthType::Pct)
        .shading(Shading::new().shd_type(ShdType::Clear).fill("E5E7EB"))
        .add_paragraph(
            Paragraph::new().add_run(Run::new().add_text(label).bold().size(20).color("111827")),
        )
}

fn summary_value_cell(value: usize) -> TableCell {
    TableCell::new().width(1000, WidthType::Pct).add_paragraph(
        Paragraph::new().add_run(
            Run::new()
                .add_text(value.to_string())
                .bold()
                .size(24)
                .color("111827"),
        ),
    )
}

fn review_status_label(status: &crate::domain::ProviderQuestionReviewStatus) -> &'static str {
    match status {
        crate::domain::ProviderQuestionReviewStatus::Correct => "OK",
        crate::domain::ProviderQuestionReviewStatus::NeedsModification => "Modificar",
        crate::domain::ProviderQuestionReviewStatus::Missing => "No aparece",
        crate::domain::ProviderQuestionReviewStatus::Pending => "Pendiente",
    }
}

fn status_color(status: &str) -> &'static str {
    match status {
        "OK" => "047857",
        "Modificar" => "1D4ED8",
        "No aparece" => "B91C1C",
        _ => "6B7280",
    }
}

fn evidence_image_paragraph(path: &str) -> Option<Paragraph> {
    let path = Path::new(path);
    if !is_supported_image_path(path) {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let mut pic = std::panic::catch_unwind(|| Pic::new(&bytes)).ok()?;
    let max_width_emu = 5_200_000_u32;
    if pic.size.0 > max_width_emu && pic.size.0 > 0 {
        let ratio = max_width_emu as f64 / pic.size.0 as f64;
        let height = (pic.size.1 as f64 * ratio).round() as u32;
        pic = pic.size(max_width_emu, height);
    }
    Some(Paragraph::new().add_run(Run::new().add_image(pic)))
}

fn is_supported_image_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "webp" | "bmp" | "gif"
            )
        })
        .unwrap_or(false)
}

fn normalize_instrument_audiences(values: &[String]) -> Vec<String> {
    let mut normalized = values
        .iter()
        .map(|value| normalize_instrument_audience(value))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    normalized.sort_by(|left, right| left.cmp(right));
    normalized.dedup();
    normalized
}

fn normalize_instrument_audience(value: &str) -> String {
    value
        .trim()
        .replace('_', " ")
        .trim_start_matches(|character: char| {
            character.is_ascii_digit() || character.is_whitespace()
        })
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
