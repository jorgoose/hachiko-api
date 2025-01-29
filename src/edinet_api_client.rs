// edinet_api_client.rs
use std::path::PathBuf;
use std::error::Error;
use reqwest::Client;
use serde::de::DeserializeOwned;
use tokio::fs;

use crate::models::api::DocumentListAPIResponse;

pub struct EdinetApiClient {
    client: Client,
    subscription_key: String,
}

impl EdinetApiClient {
    pub fn new(subscription_key: String) -> Self {
        Self {
            client: Client::new(),
            subscription_key,
        }
    }

    pub async fn get_document_list(&self, date: &str) -> Result<DocumentListAPIResponse, Box<dyn Error>> {
        let url = format!(
            "https://api.edinet-fsa.go.jp/api/v2/documents.json?date={}&type=2&Subscription-Key={}",
            date, self.subscription_key
        );

        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err("Failed to fetch document list".into());
        }

        let api_resp = response.json::<DocumentListAPIResponse>().await?;
        Ok(api_resp)
    }

    pub async fn download_xbrl(
        &self,
        doc_id: &str,
        base_dir: &PathBuf,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let url = format!(
            "https://api.edinet-fsa.go.jp/api/v2/documents/{}?type=1&Subscription-Key={}",
            doc_id, self.subscription_key
        );

        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            return Ok(None);
        }

        let doc_dir = base_dir.join("xbrl");
        fs::create_dir_all(&doc_dir).await?;
        
        let filename = format!("{}_{}.zip", doc_id, "xbrl");
        let file_path = doc_dir.join(&filename);
        
        let bytes = response.bytes().await?;
        fs::write(&file_path, &bytes).await?;
        
        Ok(Some(format!("xbrl/{}", filename)))
    }
}