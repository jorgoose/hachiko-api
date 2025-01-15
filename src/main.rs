use std::env;
use std::error::Error;
use tokio::time::{sleep, Duration};

use chrono::{NaiveDate, Duration as ChronoDuration};
use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

// ------------------
// Data structures matching EDINET's JSON
// ------------------

#[derive(Deserialize, Debug)]
struct DocumentListAPIResponse {
    metadata: DocumentListMetadata,
    results: Vec<DocumentInfo>,
}

#[derive(Deserialize, Debug)]
struct DocumentListMetadata {
    title: String,
    status: String,
    message: String,
    #[serde(rename = "processDateTime")]
    process_date: String,
    #[serde(rename = "resultset")]
    result_set: ResultSet,
}

#[derive(Deserialize, Debug)]
struct ResultSet {
    count: i32,
}

#[derive(Deserialize, Debug)]
struct DocumentInfo {
    #[serde(rename = "seqNumber")]
    seq_number: i32,
    #[serde(rename = "docID")]
    doc_id: String,
    #[serde(rename = "edinetCode")]
    edinet_code: Option<String>,
    #[serde(rename = "secCode")]
    sec_code: Option<String>,
    #[serde(rename = "docTypeCode")]
    doc_type_code: Option<String>,
    #[serde(rename = "submitDateTime")]
    submit_date_time: Option<String>,
    // ... add more fields as needed
}

// Add new struct for storing quarterly reports
#[derive(Serialize, Deserialize, Debug)]
struct QuarterlyReport {
    date: String,
    doc_id: String,
    sec_code: Option<String>,
    doc_type_code: String,
    submit_date_time: Option<String>,
    edinet_code: Option<String>,
}

// ------------------
// Async main
// ------------------
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let subscription_key =
        env::var("EDINET_API_KEY").expect("Please set EDINET_API_KEY in .env or environment.");

    // 3. from_ymd_opt => returns Option<NaiveDate>; unwrap or expect if valid
    let start_date = NaiveDate::from_ymd_opt(2015, 4, 1)
        .expect("Invalid date (2015-04-01). Check your from_ymd_opt() usage.");

    let end_date = start_date + ChronoDuration::days(30);

    // 4. Create req
    let client = Client::new();

    let mut quarterly_reports = Vec::new();

    let mut current_date = start_date;
    while current_date < end_date {
        let date_str = current_date.format("%Y-%m-%d").to_string();

        let url = format!(
            "https://api.edinet-fsa.go.jp/api/v2/documents.json?date={}&type=2&Subscription-Key={}",
            date_str, subscription_key
        );

        match client.get(&url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<DocumentListAPIResponse>().await {
                        Ok(api_resp) => {
                            // Just to "use" the never-read fields:
                            println!(
                                "Metadata => Title: {}, ProcessDate: {}",
                                api_resp.metadata.title, api_resp.metadata.process_date
                            );

                            // Check "status" in JSON
                            if api_resp.metadata.status == "200" {
                                let count = api_resp.metadata.result_set.count;
                                if count > 0 {
                                    for doc in api_resp.results {
                                        if let Some(code) = &doc.doc_type_code {
                                            if code == "140" || code == "150" {
                                                quarterly_reports.push(QuarterlyReport {
                                                    date: date_str.clone(),
                                                    doc_id: doc.doc_id,
                                                    sec_code: doc.sec_code,
                                                    doc_type_code: code.to_string(),
                                                    submit_date_time: doc.submit_date_time,
                                                    edinet_code: doc.edinet_code,
                                                });
                                            }
                                        }
                                    }
                                } else {
                                    println!("[{}] No documents found.", date_str);
                                }
                            } else {
                                eprintln!(
                                    "API returned status={} for date={} => {}",
                                    api_resp.metadata.status,
                                    date_str,
                                    api_resp.metadata.message
                                );
                            }
                        }
                        Err(json_err) => {
                            eprintln!("Error parsing JSON on {}: {:?}", date_str, json_err);
                        }
                    }
                } else {
                    eprintln!(
                        "HTTP request for {} returned error status: {}",
                        date_str,
                        resp.status()
                    );
                }
            }
            Err(req_err) => {
                eprintln!("Error sending request for {}: {:?}", date_str, req_err);
            }
        }

        // Instead of current_date.succ(), we add 1 day
        current_date = current_date + ChronoDuration::days(1);

        // Sleep ~250ms
        sleep(Duration::from_millis(250)).await;
    }

    // After the main loop ends, save to file
    let json = serde_json::to_string_pretty(&quarterly_reports)?;
    let mut file = File::create("quarterly_reports.json")?;
    file.write_all(json.as_bytes())?;
    println!("Saved {} quarterly reports to quarterly_reports.json", quarterly_reports.len());

    Ok(())
}
