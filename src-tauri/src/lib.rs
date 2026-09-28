use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader},
};
use std::fs;

use csv::{ReaderBuilder};
use encoding_rs::WINDOWS_1252;
use regex::Regex;

fn extract_student_nip_codes(content: &str) -> Vec<String> {
    let student_nip_code = Regex::new(r"\d{8}").unwrap();

    student_nip_code
        .find_iter(content)
        .map(|m| m.as_str().to_string())
        .collect()
}

// fn parse_scodoc_grades(
//     path: impl AsRef<Path>,
//     columns_to_keep: &HashSet<String>,
//     re_ue: &Regex,
// ) -> Result<String> {
//     let mut rdr = csv::Reader::from_path(path);
//     for result in rdr.records() {
//         // The iterator yields Result<StringRecord, Error>, so we check the
//         // error here.
//         let record = result?;
//         println!("{:?}", record);
//     }
//     Ok("Hello")
//     // let mut rdr = ReaderBuilder::new()
//     //     .has_headers(true)
//     //     .from_path(path)
//     //     .map_err(|e| e.to_string())?;

//     // // Skip the first line
//     // let mut skipped = csv::StringRecord::new();
//     // rdr.read_record(&mut skipped)
//     //     .map_err(|e| e.to_string())?;

//     // let headers = rdr
//     //     .headers()
//     //     .map_err(|e| e.to_string())?
//     //     .clone();

//     // rdr.records()
//     //     .map(|result| {
//     //         let record = result.map_err(|e| e.to_string())?;

//     //         Ok(headers
//     //             .iter()
//     //             .zip(record.iter())
//     //             // .filter(|(key, _)| {
//     //             //     columns_to_keep.contains(*key) || re_ue.is_match(key)
//     //             // })
//     //             .map(|(key, value)| (key.to_owned(), value.to_owned()))
//     //             .collect())
//     //     })
//     //     .collect()
// }

pub fn parse_scodoc_csv(
    export_scodoc: &str,
) -> Result<Vec<HashMap<String, String>>, String> {
    let columns_to_keep: HashSet<&str> =
        ['code_nip', 'Nom', 'Prénom'].into_iter().collect();

    let regex_ue = Regex::new(r"^UE")
        .map_err(|e| e.to_string())?;

    // Open file
    let file = File::open(export_scodoc)
        .map_err(|e| e.to_string())?;

    let mut buf_reader = BufReader::new(file);

    // Skip first line
    let mut first_line = String::new();
    buf_reader
        .read_line(&mut first_line)
        .map_err(|e| e.to_string())?;

    // The second line is now the CSV header
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(buf_reader);

    let headers = reader
        .headers()
        .map_err(|e| e.to_string())?
        .clone();

    let mut filtered = Vec::new();

    for record in reader.records() {
        let record = record.map_err(|e| e.to_string())?;

        let row = headers
            .iter()
            .zip(record.iter())
            .filter(|(key, _)| {
                columns_to_keep.contains(key)
                    || regex_ue.is_match(key)
            })
            .map(|(key, value)| {
                (key.to_string(), value.to_string())
            })
            .collect::<HashMap<String, String>>();

        filtered.push(row);
    }

    Ok(filtered)
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

    let content = match String::from_utf8(bytes.clone()) {
        Ok(content) => content,
        Err(_) => {
            let (content, _, _) = WINDOWS_1252.decode(&bytes);
            content.into_owned()
        }
    };

    let nip_codes: Vec<String> = extract_student_nip_codes(&content);

    // let columns_to_keep = HashSet::from([
    //     "code_nip".to_string(),
    //     "Nom".to_string(),
    //     "Prénom".to_string(),
    // ]);

    // let re_ue = Regex::new(r"^UE\d+").unwrap();

    // let mut rdr = csv::Reader::from_path(export_scodoc).map_err(|e| e.to_string())?;

    // let headers = rdr.headers()?.clone();

    // for result in rdr.records() {
    //     let record = result.map_err(|e| e.to_string())?;
    //     println!("{:?}", record);
    // }

    let filtered = parse_scodoc_csv(&export_scodoc)?;

    println!("{:#?}", filtered.get(0));
    println!("{:#?}", filtered.get(1));


    // let filtered = parse_scodoc_grades(export_scodoc, &columns_to_keep, &re_ue)?;

    // println!("File content:\n{}", content);
    // for nip in &nip_codes {
    // println!("{:#?}", filtered);
    // }

    Ok(format!(
        "{0}, this is {1}. {1}, this is {0}",
        "Alice", "Helo"
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![process_form_payload])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
