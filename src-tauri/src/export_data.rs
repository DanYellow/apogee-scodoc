use indexmap::IndexMap;
use std::{
    fs::File,
    io::{Write},
};
use umya_spreadsheet::{self, writer};

use csv::{WriterBuilder};

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

pub fn export_xlsx(result: &[IndexMap<String, String>], output_path: &str) -> Result<(), String> {
    if result.is_empty() {
        return Ok(());
    }

    let mut book = umya_spreadsheet::new_file();
    let sheet = book.active_sheet_mut();

    // Headers
    let headers: Vec<&String> = result[0].keys().collect();

    for (col, header) in headers.iter().enumerate() {
        sheet
            .cell_mut(((col + 1) as u32, 1))
            .set_value(header.as_str());
    }

    // Rows
    for (row_idx, row) in result.iter().enumerate() {
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

    writer::xlsx::write(&book, output_path).map_err(|e| e.to_string())?;

    Ok(())
}