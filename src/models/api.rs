use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct DocumentListAPIResponse {
    pub metadata: DocumentListMetadata,
    pub results: Vec<DocumentInfo>,
}

#[derive(Deserialize, Debug)]
pub struct DocumentListMetadata {
    pub title: String,
    pub status: String,
    pub message: String,
    #[serde(rename = "processDateTime")]
    pub process_date: String,
    #[serde(rename = "resultset")]
    pub result_set: ResultSet,
}

#[derive(Deserialize, Debug)]
pub struct ResultSet {
    pub count: i32,
}

#[derive(Deserialize, Debug)]
pub struct DocumentInfo {
    #[serde(rename = "seqNumber")]
    pub seq_number: i32,
    #[serde(rename = "docID")]
    pub doc_id: String,
    #[serde(rename = "edinetCode")]
    pub edinet_code: Option<String>,
    #[serde(rename = "secCode")]
    pub sec_code: Option<String>,
    #[serde(rename = "docTypeCode")]
    pub doc_type_code: Option<String>,
    #[serde(rename = "submitDateTime")]
    pub submit_date_time: Option<String>,
    #[serde(rename = "filerName")]
    pub filer_name: Option<String>,
}