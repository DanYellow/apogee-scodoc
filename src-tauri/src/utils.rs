use regex::Regex;

pub fn extract_student_nip_codes(content: &str) -> Vec<String> {
    let student_nip_code = Regex::new(r"\d{8}").unwrap();

    student_nip_code
        .find_iter(content)
        .map(|m| m.as_str().to_string())
        .collect()
}

pub fn extract_column_for_index(csv_data: String, row_index: usize) -> Result<Vec<String>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b',')
        .from_reader(csv_data.as_bytes());

    let mut column = Vec::new();

    for result in reader.records() {
        let record = result.map_err(|e| e.to_string())?;

        if let Some(value) = record.get(row_index) {
            column.push(value.to_string());
        }
    }

    Ok(column)
}

// let values = get_first_column(&csv_data, b',')? u8;
