use regex::Regex;

pub fn extract_student_nip_codes(content: &str) -> Vec<String> {
    let student_nip_code = Regex::new(r"\d{8}").unwrap();

    student_nip_code
        .find_iter(content)
        .map(|m| m.as_str().to_string())
        .collect()
}