use super::{DynamicXBRLContent, IncomeStatement, BalanceSheet};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct QuarterlyReport {
    pub date: String,
    pub doc_id: String,
    pub sec_code: Option<String>,
    pub doc_type_code: String,
    pub submit_date_time: Option<String>,
    pub edinet_code: Option<String>,
    pub filer_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xbrl_zip_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xbrl_content: Option<DynamicXBRLContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub income_statement: Option<IncomeStatement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_sheet: Option<BalanceSheet>,
}