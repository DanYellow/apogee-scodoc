use indexmap::IndexMap;
use std::fs;
use std::{
    collections::HashSet,
    io::{BufRead, BufReader},
};
use umya_spreadsheet::{self};

use csv::ReaderBuilder;
use encoding_rs::WINDOWS_1252;
use regex::Regex;

mod export_data;
mod utils;
pub mod models;

fn parse_scodoc_csv(csv_data: String) -> Result<Vec<IndexMap<String, String>>, String> {
    let columns_to_keep: HashSet<&str> = ["code_nip", "Nom", "Prénom"].into_iter().collect();

    let regex_ue = Regex::new(r"^UE\d").map_err(|e| e.to_string())?;

    let mut buf_reader = BufReader::new(csv_data.as_bytes());

    // Skip first line
    let mut first_line = String::new();
    buf_reader
        .read_line(&mut first_line)
        .map_err(|e| e.to_string())?;

    // The second line is now the CSV header
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        // .delimiter(delimiter)
        .from_reader(buf_reader);

    let headers = reader.headers().map_err(|e| e.to_string())?.clone();

    let mut filtered = Vec::new();

    for record in reader.records() {
        let record = record.map_err(|e| e.to_string())?;

        let row = headers
            .iter()
            .zip(record.iter())
            .filter(|(key, _)| columns_to_keep.contains(key) || regex_ue.is_match(key))
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect::<IndexMap<String, String>>();

        filtered.push(row);
    }

    filtered.sort_by(|a, b| {
        let name_a = a.get("Nom").map(String::as_str).unwrap_or("");
        let name_b = b.get("Nom").map(String::as_str).unwrap_or("");

        name_a.cmp(name_b)
    });

    filtered.retain(|row| row.get("Nom").map_or(false, |value| !value.is_empty()));

    Ok(filtered)
}

fn sheet_to_csv(sheet: &umya_spreadsheet::Worksheet) -> String {
    let (max_col, max_row) = sheet.highest_column_and_row();
    let mut csv = String::new();

    for row in 1..=max_row {
        for col in 1..=max_col {
            if col > 1 {
                csv.push(',');
            }

            let value = sheet
                .cell((col, row))
                .map(|cell| cell.value().into_owned())
                .unwrap_or_default();

            if value.contains([',', '"', '\n', '\r']) {
                csv.push('"');
                csv.push_str(&value.replace('"', "\"\""));
                csv.push('"');
            } else {
                csv.push_str(&value);
            }
        }

        csv.push('\n');
    }

    csv
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[tauri::command]
fn process_form_payload(
    export_apogee: &str,
    export_scodoc: &str,
    bareme: &str,
) -> Result<String, String> {
    let bytes =
        fs::read(export_apogee).map_err(|e| format!("Failed to read {}: {}", export_apogee, e))?;

    let export_apogee_content = match String::from_utf8(bytes.clone()) {
        Ok(content) => content,
        Err(_) => {
            let (content, _, _) = WINDOWS_1252.decode(&bytes);
            content.into_owned()
        }
    };

    let nip_codes: Vec<String> = utils::extract_student_nip_codes(&export_apogee_content);
    let mut list_resigning_students: Vec<String> = Vec::new();

    let csv_data: String;
    let path = std::path::Path::new(export_scodoc);
    if path
        .extension()
        .map(|s| s.to_ascii_lowercase() == "xlsx")
        .unwrap_or(false)
    {
        let book = umya_spreadsheet::reader::xlsx::read(path).unwrap();

        let sheet = book.sheet(0).unwrap();
        csv_data = sheet_to_csv(sheet);

        if book.sheet(1).is_ok() {
            let sheet_resigning_students = book.sheet(1).unwrap();
            list_resigning_students =
                utils::extract_column_for_index(sheet_to_csv(sheet_resigning_students), 0)
                    .unwrap_or_default();
        }
    } else {
        csv_data = std::fs::read_to_string(export_scodoc).map_err(|e| e.to_string())?;
    }

    let filtered_scodoc_data = parse_scodoc_csv(csv_data)?;

    let not_found_students: Vec<_> = filtered_scodoc_data
        .iter()
        .filter(|row| {
            row.get("code_nip")
                .is_some_and(|nip_student| !nip_codes.contains(nip_student))
        })
        .cloned()
        .collect();

    let found_students: Vec<_> = filtered_scodoc_data
        .iter()
        .filter(|row| {
            row.get("code_nip")
                .is_some_and(|nip_student| nip_codes.contains(nip_student))
        })
        .cloned()
        .collect();

    let bareme_valeur = if bareme.is_empty() { "20" } else { bareme };
    let data_for_export = export_data::compute_data_for_export(
        found_students,
        bareme_valeur,
        list_resigning_students,
    )?;

    let json_data = serde_json::json!({
        "found_students": data_for_export,
        "not_found_students": not_found_students,
    });

    let json_string = serde_json::to_string(&json_data).map_err(|e| e.to_string())?;

    Ok(json_string)
}

#[tauri::command]
fn download_data(output_path: &str, csv_content: &str, output_type: String) -> Result<(), String> {
    let data: models::PayloadExportJsonData =
        serde_json::from_str(csv_content).map_err(|e| e.to_string())?;
    if output_type == "xlsx" {
        export_data::export_xlsx(data, &output_path)?;
    } else {
        export_data::export_csv(&data.found_students, &output_path)?;
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            process_form_payload,
            download_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
