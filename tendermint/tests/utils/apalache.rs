use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct ApalacheTestBatch {
    pub description: String,
    pub model: String,
    pub length: Option<u64>,
    pub timeout: Option<u64>,
    pub tests: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ApalacheTestCase {
    pub model: String,
    pub test: String,
    pub length: Option<u64>,
    pub timeout: Option<u64>,
}
