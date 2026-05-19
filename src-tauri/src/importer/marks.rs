use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader as XmlReader;
use zip::ZipArchive;

use crate::error::AppError;

#[derive(Debug, Default, Clone)]
pub struct SheetMarks {
    pub red: BTreeSet<(usize, usize)>,
    pub blue: BTreeSet<(usize, usize)>,
    pub green: BTreeSet<(usize, usize)>,
}

#[derive(Debug, Default, Clone, Copy)]
struct StyleMark {
    red: bool,
    blue: bool,
    green: bool,
}

pub fn xlsx_marked_cells_by_sheet(path: &Path) -> Result<HashMap<String, SheetMarks>, AppError> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !matches!(extension.as_str(), "xlsx" | "xlsm") {
        return Ok(HashMap::new());
    }

    let file = File::open(path)?;
    let mut zip = ZipArchive::new(file)
        .map_err(|error| AppError::Validation(format!("Could not open XLSX archive: {error}")))?;
    let style_marks = parse_style_marks(&read_zip_text(&mut zip, "xl/styles.xml")?)?;
    let sheet_paths = parse_workbook_sheet_paths(
        &read_zip_text(&mut zip, "xl/workbook.xml")?,
        &read_zip_text(&mut zip, "xl/_rels/workbook.xml.rels")?,
    )?;
    let mut marked_cells = HashMap::new();

    for (sheet_name, sheet_path) in sheet_paths {
        let Ok(xml) = read_zip_text(&mut zip, &sheet_path) else {
            continue;
        };
        marked_cells.insert(sheet_name, parse_marked_cells(&xml, &style_marks)?);
    }

    Ok(marked_cells)
}

#[cfg(test)]
pub fn xlsx_red_cells_by_sheet(
    path: &Path,
) -> Result<HashMap<String, BTreeSet<(usize, usize)>>, AppError> {
    Ok(xlsx_marked_cells_by_sheet(path)?
        .into_iter()
        .map(|(sheet, marks)| (sheet, marks.red))
        .collect())
}

fn read_zip_text(zip: &mut ZipArchive<File>, path: &str) -> Result<String, AppError> {
    let mut file = zip.by_name(path).map_err(|error| {
        AppError::Validation(format!("Could not read XLSX part {path}: {error}"))
    })?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    Ok(text)
}

fn parse_style_marks(styles_xml: &str) -> Result<Vec<StyleMark>, AppError> {
    let mut reader = XmlReader::from_str(styles_xml);
    reader.config_mut().trim_text(true);
    let mut fonts = Vec::new();
    let mut fills = Vec::new();
    let mut cell_xfs = Vec::new();
    let mut section: Option<&'static str> = None;
    let mut current_font = StyleMark::default();
    let mut current_fill = StyleMark::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => match event.local_name().as_ref() {
                b"fonts" => section = Some("fonts"),
                b"fills" => section = Some("fills"),
                b"cellXfs" => section = Some("cellXfs"),
                b"font" if section == Some("fonts") => current_font = StyleMark::default(),
                b"fill" if section == Some("fills") => current_fill = StyleMark::default(),
                b"color" if section == Some("fonts") => {
                    current_font = merge_mark(current_font, event_mark(&event));
                }
                b"fgColor" | b"bgColor" if section == Some("fills") => {
                    current_fill = merge_mark(current_fill, event_mark(&event));
                }
                b"xf" if section == Some("cellXfs") => cell_xfs.push(parse_xf(&event)),
                _ => {}
            },
            Ok(Event::Empty(event)) => match event.local_name().as_ref() {
                b"color" if section == Some("fonts") => {
                    current_font = merge_mark(current_font, event_mark(&event));
                }
                b"fgColor" | b"bgColor" if section == Some("fills") => {
                    current_fill = merge_mark(current_fill, event_mark(&event));
                }
                b"xf" if section == Some("cellXfs") => cell_xfs.push(parse_xf(&event)),
                _ => {}
            },
            Ok(Event::End(event)) => match event.local_name().as_ref() {
                b"font" if section == Some("fonts") => fonts.push(current_font),
                b"fill" if section == Some("fills") => fills.push(current_fill),
                b"fonts" | b"fills" | b"cellXfs" => section = None,
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(error) => {
                return Err(AppError::Validation(format!(
                    "Invalid XLSX styles XML: {error}"
                )))
            }
            _ => {}
        }
    }

    Ok(cell_xfs
        .into_iter()
        .map(|(fill_id, font_id)| {
            merge_mark(
                fills.get(fill_id).copied().unwrap_or_default(),
                fonts.get(font_id).copied().unwrap_or_default(),
            )
        })
        .collect())
}

fn parse_workbook_sheet_paths(
    workbook_xml: &str,
    rels_xml: &str,
) -> Result<HashMap<String, String>, AppError> {
    let mut rel_targets = HashMap::new();
    let mut rel_reader = XmlReader::from_str(rels_xml);
    rel_reader.config_mut().trim_text(true);
    loop {
        match rel_reader.read_event() {
            Ok(Event::Empty(event)) | Ok(Event::Start(event))
                if event.local_name().as_ref() == b"Relationship" =>
            {
                let id = attr_value(&event, b"Id").unwrap_or_default();
                let target = attr_value(&event, b"Target").unwrap_or_default();
                if !id.is_empty() && !target.is_empty() {
                    rel_targets.insert(id, normalize_xlsx_part_path(&target));
                }
            }
            Ok(Event::Eof) => break,
            Err(error) => {
                return Err(AppError::Validation(format!(
                    "Invalid XLSX rels XML: {error}"
                )))
            }
            _ => {}
        }
    }

    let mut sheets = HashMap::new();
    let mut workbook_reader = XmlReader::from_str(workbook_xml);
    workbook_reader.config_mut().trim_text(true);
    loop {
        match workbook_reader.read_event() {
            Ok(Event::Empty(event)) | Ok(Event::Start(event))
                if event.local_name().as_ref() == b"sheet" =>
            {
                let name = attr_value(&event, b"name").unwrap_or_default();
                let relationship_id = namespaced_attr_value(&event, b"id").unwrap_or_default();
                if let Some(target) = rel_targets.get(&relationship_id) {
                    sheets.insert(name, target.clone());
                }
            }
            Ok(Event::Eof) => break,
            Err(error) => {
                return Err(AppError::Validation(format!(
                    "Invalid XLSX workbook XML: {error}"
                )))
            }
            _ => {}
        }
    }
    Ok(sheets)
}

fn parse_marked_cells(
    worksheet_xml: &str,
    style_marks: &[StyleMark],
) -> Result<SheetMarks, AppError> {
    let mut reader = XmlReader::from_str(worksheet_xml);
    reader.config_mut().trim_text(true);
    let mut marked_cells = SheetMarks::default();
    loop {
        match reader.read_event() {
            Ok(Event::Empty(event)) | Ok(Event::Start(event))
                if event.local_name().as_ref() == b"c" =>
            {
                let style_index = attr_value(&event, b"s")
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(0);
                let mark = style_marks.get(style_index).copied().unwrap_or_default();
                if mark.red || mark.blue || mark.green {
                    if let Some(reference) = attr_value(&event, b"r") {
                        if let Some(cell) = cell_reference_to_position(&reference) {
                            if mark.red {
                                marked_cells.red.insert(cell);
                            }
                            if mark.blue {
                                marked_cells.blue.insert(cell);
                            }
                            if mark.green {
                                marked_cells.green.insert(cell);
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(error) => {
                return Err(AppError::Validation(format!(
                    "Invalid XLSX worksheet XML: {error}"
                )))
            }
            _ => {}
        }
    }
    Ok(marked_cells)
}

fn parse_xf(event: &BytesStart<'_>) -> (usize, usize) {
    let fill_id = attr_value(event, b"fillId")
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let font_id = attr_value(event, b"fontId")
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    (fill_id, font_id)
}

fn event_mark(event: &BytesStart<'_>) -> StyleMark {
    attr_value(event, b"rgb")
        .map(|value| rgb_mark(&value))
        .unwrap_or_default()
}

fn merge_mark(left: StyleMark, right: StyleMark) -> StyleMark {
    StyleMark {
        red: left.red || right.red,
        blue: left.blue || right.blue,
        green: left.green || right.green,
    }
}

fn rgb_mark(value: &str) -> StyleMark {
    let normalized = value.trim().trim_start_matches('#');
    let rgb = if normalized.len() >= 6 {
        &normalized[(normalized.len() - 6)..]
    } else {
        normalized
    };
    let Ok(number) = u32::from_str_radix(rgb, 16) else {
        return StyleMark::default();
    };
    let red = (number >> 16) & 0xff;
    let green = (number >> 8) & 0xff;
    let blue = number & 0xff;
    StyleMark {
        red: red >= 180 && green <= 120 && blue <= 120,
        blue: blue >= 180 && red <= 170 && green <= 230,
        green: green >= 150 && red <= 210 && blue <= 180,
    }
}

fn attr_value(event: &BytesStart<'_>, name: &[u8]) -> Option<String> {
    event.attributes().flatten().find_map(|attribute| {
        if attribute.key.as_ref() == name {
            Some(String::from_utf8_lossy(attribute.value.as_ref()).into_owned())
        } else {
            None
        }
    })
}

fn namespaced_attr_value(event: &BytesStart<'_>, local_name: &[u8]) -> Option<String> {
    event.attributes().flatten().find_map(|attribute| {
        let key = attribute.key.as_ref();
        if key == local_name || key.ends_with(local_name) {
            Some(String::from_utf8_lossy(attribute.value.as_ref()).into_owned())
        } else {
            None
        }
    })
}

fn normalize_xlsx_part_path(target: &str) -> String {
    let target = target.trim_start_matches('/');
    if target.starts_with("xl/") {
        target.to_string()
    } else {
        format!("xl/{target}")
    }
}

fn cell_reference_to_position(reference: &str) -> Option<(usize, usize)> {
    let mut column = 0usize;
    let mut row = String::new();
    for character in reference.chars() {
        if character.is_ascii_alphabetic() {
            column = column * 26 + (character.to_ascii_uppercase() as usize - 'A' as usize + 1);
        } else if character.is_ascii_digit() {
            row.push(character);
        }
    }
    if column == 0 || row.is_empty() {
        return None;
    }
    Some((row.parse().ok()?, column - 1))
}
