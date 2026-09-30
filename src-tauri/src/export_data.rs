use indexmap::IndexMap;
use regex::Regex;
use std::{fs::File, io::Write, path::Path};

use csv::WriterBuilder;
use umya_spreadsheet::{self, writer};

use crate::models;

pub fn export_csv(result: &[IndexMap<String, String>], output_path: &str) -> Result<(), String> {
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

fn sheet_name(key: &str) -> &str {
    match key {
        "found_students" => "Données pour Apogée",
        "not_found_students" => "Etudiants non trouvés",
        _ => key,
    }
}

pub fn export_xlsx(result: models::PayloadExportJsonData, output_path: &str) -> Result<(), String> {
    let output_path = Path::new(output_path);

    let output_path = if output_path.extension().is_none() {
        output_path.with_extension("xlsx")
    } else {
        output_path.to_path_buf()
    };

    let mut book = umya_spreadsheet::new_file();
    let _ = book.remove_sheet(0);

    let result_json = serde_json::to_value(&result).map_err(|e| e.to_string())?;

    if let Some(object) = result_json.as_object() {
        for (key, value) in object {
            let list_students: Vec<IndexMap<String, String>> =
                serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
            let tab_name = sheet_name(key);
            let sheet = book.new_sheet(tab_name).map_err(|e| e.to_string())?;

            if list_students.is_empty() {
                continue;
            }

            // Headers
            let headers: Vec<&String> = list_students[0].keys().collect();

            for (col, header) in headers.iter().enumerate() {
                sheet
                    .cell_mut(((col + 1) as u32, 1))
                    .set_value(header.as_str());
            }

            // Rows
            for (row_idx, row) in list_students.iter().enumerate() {
                for (col_idx, key) in headers.iter().enumerate() {
                    let value = row.get(*key).map(String::as_str).unwrap_or("");

                    let coordinate = ((col_idx + 1) as u32, (row_idx + 2) as u32);

                    if col_idx == 0 {
                        sheet.cell_mut(coordinate).set_value_string(value);
                    } else {
                        sheet.cell_mut(coordinate).set_value(value);
                    }
                }
            }

            sheet.insert_new_column("0", 1);

            let (last_col_idx, max_row) = (sheet.highest_column(), sheet.highest_row());

            for row_idx in 1..=max_row {
                // Get the value from the last column
                if let Some(cell) = sheet.cell((last_col_idx, row_idx)) {
                    let value = cell.value().to_string();

                    // Set it in the first column (Column 1 / 'A')
                    sheet.cell_mut((1, row_idx)).set_value(value);
                }
            }
            sheet.remove_column_by_index(sheet.highest_column(), 1);

        }
    }

    writer::xlsx::write(&book, output_path).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn compute_data_for_export(
    csv: Vec<IndexMap<String, String>>,
    bareme: &str,
    list_failed_students: Vec<String>,
) -> Result<Vec<IndexMap<String, String>>, String> {
    let regex_ue = Regex::new(r"^UE\d").map_err(|e| e.to_string())?;

    let result: Vec<IndexMap<String, String>> = csv
        .iter()
        .map(|obj| {
            let mut new_obj = IndexMap::new();

            for (key, value) in obj {
                new_obj.insert(key.clone(), value.clone());

                if regex_ue.is_match(key) {
                    let mut bareme_value = bareme.to_string();
                    let mut final_grade: String = new_obj.get(key).unwrap().clone();

                    if new_obj
                        .get("code_nip")
                        .is_some_and(|nip| list_failed_students.contains(nip))
                    {
                        bareme_value = "0".to_string();
                        final_grade = "DEF".to_string();
                    }

                    new_obj.insert(format!("{}", key), final_grade);
                    new_obj.insert(format!("{}_barème", key), bareme_value);
                    new_obj.insert(format!("{}_pts_jury", key), String::new());
                    new_obj.insert(format!("{}_résultat", key), String::new());
                }
            }

            new_obj
        })
        .collect();

    Ok(result)
}
