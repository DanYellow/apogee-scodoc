use serde::Deserialize;
use indexmap::IndexMap;

#[derive(Deserialize)]
pub struct PayloadExportJsonData {
    pub found_students: Vec<IndexMap<String, String>>,
    pub not_found_students: Vec<IndexMap<String, String>>,
}