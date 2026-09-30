use serde::{Deserialize, Serialize};
use indexmap::IndexMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct PayloadExportJsonData {
    pub found_students: Vec<IndexMap<String, String>>,
    pub not_found_students: Vec<IndexMap<String, String>>,
}