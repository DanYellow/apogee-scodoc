use indexmap::IndexMap;
use std::fs;
use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader, Write},
};

use csv::{ReaderBuilder, WriterBuilder};
use encoding_rs::WINDOWS_1252;
use regex::Regex;

fn extract_student_nip_codes(content: &str) -> Vec<String> {
    let student_nip_code = Regex::new(r"\d{8}").unwrap();

    student_nip_code
        .find_iter(content)
        .map(|m| m.as_str().to_string())
        .collect()
}

fn parse_scodoc_csv(export_scodoc: &str, delimiter: u8) -> Result<Vec<IndexMap<String, String>>, String> {
    let columns_to_keep: HashSet<&str> = ["code_nip", "Nom", "Prénom"].into_iter().collect();

    let regex_ue = Regex::new(r"^UE\d").map_err(|e| e.to_string())?;

    // Open file
    let file = File::open(export_scodoc).map_err(|e| e.to_string())?;

    let mut buf_reader = BufReader::new(file);

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

fn export_csv(result: &[IndexMap<String, String>], output_path: &str) -> Result<(), String> {
    if result.is_empty() {
        return Ok(());
    }

    let mut file = File::create(output_path).map_err(|e| e.to_string())?;
    file.write_all(b"\xEF\xBB\xBF").map_err(|e| e.to_string())?;

    let mut writer = WriterBuilder::new().delimiter(b';').from_writer(file);

    // Headers come from the first row
    let headers: Vec<&String> = result[0].keys().collect();

    writer.write_record(&headers).map_err(|e| e.to_string())?;

    // Rows
    for row in result {
        let values = headers
            .iter()
            .map(|key| row.get(*key).map(String::as_str).unwrap_or(""))
            .collect::<Vec<_>>();

        writer.write_record(values).map_err(|e| e.to_string())?;
    }

    writer.flush().map_err(|e| e.to_string())?;

    Ok(())
}

fn compute_data_for_export(
    csv: Vec<IndexMap<String, String>>,
    bareme: &str,
) -> Result<Vec<IndexMap<String, String>>, String> {
    let regex_ue = Regex::new(r"^UE\d").map_err(|e| e.to_string())?;

    let result: Vec<IndexMap<String, String>> = csv
        .iter()
        .map(|obj| {
            let mut new_obj = IndexMap::new();

            for (key, value) in obj {
                new_obj.insert(key.clone(), value.clone());

                if regex_ue.is_match(key) {
                    new_obj.insert(format!("{}_barème", key), bareme.to_string());
                    new_obj.insert(format!("{}_pts_jury", key), String::new());
                    new_obj.insert(format!("{}_résultat", key), String::new());
                }
            }

            new_obj
        })
        .collect();

    Ok(result)
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[tauri::command]
fn process_form_payload(
    export_apogee: &str,
    export_scodoc: &str,
    bareme: &str,
    separateur_csv: &str,
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

    let nip_codes: Vec<String> = extract_student_nip_codes(&export_apogee_content);
 
    let mut filtered_scodoc_data = parse_scodoc_csv(&export_scodoc, separateur_csv.as_ptr() as u8)?;

    filtered_scodoc_data.retain(|row| {
        row.get("code_nip")
            .map_or(false, |nip_student| nip_codes.contains(nip_student))
    });

    let bareme_valeur = if bareme.is_empty() { "20" } else { bareme };
    let data_for_export = compute_data_for_export(filtered_scodoc_data, bareme_valeur)?;

    // export_csv(&data_for_export, "testRRR.tmp.csv")?;

    let json_string = serde_json::to_string(&data_for_export).map_err(|e| e.to_string())?;

    Ok(json_string)
}

#[tauri::command]
fn download_data(output_path: &str, csv_content: &str) -> Result<(), String> {
    let data: Vec<IndexMap<String, String>> =
        serde_json::from_str(csv_content).map_err(|e| e.to_string())?;
    export_csv(&data, &output_path)?;

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
