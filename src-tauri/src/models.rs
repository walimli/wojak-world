use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryState {
    pub setup_complete: bool,
    pub roots: Vec<String>,
    pub default_roots: Vec<String>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanReport {
    pub seen: u64,
    pub added: u64,
    pub updated: u64,
    pub marked_missing: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageSummary {
    pub path: String,
    pub basename: String,
    pub folder: String,
    pub extension: String,
    pub size: u64,
    pub mtime: i64,
    pub favorite: bool,
    pub display_name: Option<String>,
    pub missing: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageDetail {
    #[serde(flatten)]
    pub image: ImageSummary,
    pub tags: Vec<String>,
    pub categories: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub items: Vec<ImageSummary>,
    pub visible: u64,
    pub total: u64,
    pub favorites: u64,
}

pub fn default_roots() -> Vec<String> {
    vec![
        r"C:\Users\William\Wojak-World\All Wojaks".to_string(),
        r"C:\Users\William\Wojak-World\Wojakland Archive".to_string(),
    ]
}
