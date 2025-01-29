use std::env;
use std::error::Error;
use std::path::PathBuf;
use tokio::time::{sleep, Duration};
use chrono::{NaiveDate, Duration as ChronoDuration};
use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::collections::HashMap;
use serde_json::Value as JsonValue;
use std::io::{Read, Write};
use zip::ZipArchive;
use quick_xml::Reader;
use quick_xml::events::Event;
use rusqlite::{params, Connection};

mod models;
mod edinet_api_client;

use models::xbrl::{XBRLElement, DynamicXBRLContent};
use models::financial_statements::{IncomeStatement, BalanceSheet};
use edinet_api_client::EdinetApiClient;

fn extract_income_statement(xbrl: &DynamicXBRLContent) -> IncomeStatement {
    let mut stmt = IncomeStatement::default();

    fn set_value_if_match(stmt: &mut IncomeStatement, element_name: &str, val: f64) {
        match element_name {
            "jppfs_cor:NetSales" => stmt.net_sales = Some(val),
            "jppfs_cor:CostOfSales" => stmt.cost_of_sales = Some(val),
            "jppfs_cor:GrossProfit" => stmt.gross_profit = Some(val),
            "jppfs_cor:SellingGeneralAndAdministrativeExpenses" => stmt.selling_general_admin = Some(val),
            "jppfs_cor:OperatingIncome" => stmt.operating_income = Some(val),
            "jppfs_cor:InterestIncomeNOI" => stmt.interest_income_noi = Some(val),
            "jppfs_cor:DividendsIncomeNOI" => stmt.dividends_income_noi = Some(val),
            "jppfs_cor:InterestAndDividendsIncomeNOI" => stmt.interest_and_dividends_income_noi = Some(val),
            "jppfs_cor:PurchaseDiscountsNOI" => stmt.purchase_discounts_noi = Some(val),
            "jppfs_cor:RentIncomeNOI" => stmt.rent_income_noi = Some(val),
            "jppfs_cor:HouseRentIncomeNOI" => stmt.house_rent_income_noi = Some(val),
            "jppfs_cor:OtherNOI" => stmt.other_noi = Some(val),
            "jppfs_cor:NonOperatingIncome" => stmt.non_operating_income = Some(val),
            "jppfs_cor:SalesDiscountsNOE" => stmt.sales_discounts_noe = Some(val),
            "jppfs_cor:RentCostOfRealEstateNOE" => stmt.rent_cost_real_estate_noe = Some(val),
            "jppfs_cor:OtherNOE" => stmt.other_noe = Some(val),
            "jppfs_cor:NonOperatingExpenses" => stmt.non_operating_expenses = Some(val),
            "jppfs_cor:OrdinaryIncome" => stmt.ordinary_income = Some(val),
            "jppfs_cor:GainOnSalesOfNoncurrentAssetsEI" => stmt.gain_on_sales_of_noncurrent_assets_ei = Some(val),
            "jppfs_cor:ExtraordinaryIncome" => stmt.extraordinary_income = Some(val),
            "jppfs_cor:IncomeBeforeIncomeTaxes" => stmt.income_before_income_taxes = Some(val),
            "jppfs_cor:IncomeTaxesCurrent" => stmt.income_taxes_current = Some(val),
            "jppfs_cor:IncomeTaxesDeferred" => stmt.income_taxes_deferred = Some(val),
            "jppfs_cor:IncomeTaxes" => stmt.income_taxes = Some(val),
            "jppfs_cor:IncomeBeforeMinorityInterests" => stmt.income_before_minority_interests = Some(val),
            "jppfs_cor:NetIncome" => stmt.net_income = Some(val),
            _ => {}
        }
    }

    fn visit_element(ele: &XBRLElement, stmt: &mut IncomeStatement) {
        if let Some(ctx) = &ele.context_ref {
            if ctx == "CurrentYTDDuration" {
                if let Some(txt) = &ele.value {
                    if let Ok(parsed_val) = txt.parse::<f64>() {
                        set_value_if_match(stmt, &ele.name, parsed_val);
                    }
                }
            }
        }

        for child in &ele.children {
            visit_element(child, stmt);
        }
    }

    for e in &xbrl.elements {
        visit_element(e, &mut stmt);
    }

    stmt
}

fn extract_balance_sheet(xbrl: &DynamicXBRLContent) -> BalanceSheet {
    let mut sheet = BalanceSheet::default();

    fn set_value_if_match(sheet: &mut BalanceSheet, element_name: &str, val: f64) {
        match element_name {
            "jppfs_cor:CashAndDeposits" => sheet.assets.cash_and_deposits = Some(val),
            "jppfs_cor:NotesAndAccountsReceivableTrade" => sheet.assets.notes_and_accounts_receivable_trade = Some(val),
            "jppfs_cor:ShortTermInvestmentSecurities" => sheet.assets.short_term_investment_securities = Some(val),
            "jppfs_cor:Merchandise" => sheet.assets.merchandise = Some(val),
            "jppfs_cor:PropertyPlantAndEquipment" => sheet.assets.property_plant_and_equipment = Some(val),
            "jppfs_cor:IntangibleAssets" => sheet.assets.intangible_assets = Some(val),
            "jppfs_cor:InvestmentsAndOtherAssets" => sheet.assets.investments_and_other_assets = Some(val),
            "jppfs_cor:Assets" => sheet.assets.total_assets = Some(val),
            
            "jppfs_cor:CurrentLiabilities" => sheet.liabilities.current_liabilities = Some(val),
            "jppfs_cor:NoncurrentLiabilities" => sheet.liabilities.noncurrent_liabilities = Some(val),
            "jppfs_cor:Liabilities" => sheet.liabilities.total_liabilities = Some(val),
            
            "jppfs_cor:ShareholdersEquity" => sheet.equity.shareholders_equity = Some(val),
            "jppfs_cor:ValuationAndTranslationAdjustments" => sheet.equity.valuation_and_translation_adjustments = Some(val),
            "jppfs_cor:NetAssets" => sheet.equity.total_equity = Some(val),
            _ => {}
        }
    }

    fn visit_element(ele: &XBRLElement, sheet: &mut BalanceSheet) {
        if let Some(ctx) = &ele.context_ref {
            if ctx == "CurrentQuarterInstant" {
                if let Some(txt) = &ele.value {
                    if let Ok(parsed_val) = txt.parse::<f64>() {
                        set_value_if_match(sheet, &ele.name, parsed_val);
                    }
                }
            }
        }

        for child in &ele.children {
            visit_element(child, sheet);
        }
    }

    for e in &xbrl.elements {
        visit_element(e, &mut sheet);
    }

    sheet
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let subscription_key =
        env::var("EDINET_API_KEY").expect("Please set EDINET_API_KEY in .env or environment.");
    let api_client = EdinetApiClient::new(subscription_key);

    let start_date = NaiveDate::from_ymd_opt(2015, 4, 1).expect("Invalid date.");
    let end_date = start_date + ChronoDuration::days(5);

    let base_dir = PathBuf::from("edinet_documents");
    fs::create_dir_all(&base_dir)?;

    let db_path = base_dir.join("reports.db");
    let conn = Connection::open(&db_path)?;

    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS quarterly_reports (
            doc_id TEXT PRIMARY KEY,
            date TEXT NOT NULL,
            sec_code TEXT,
            doc_type_code TEXT NOT NULL,
            submit_date_time TEXT,
            edinet_code TEXT,
            filer_name TEXT,
            xbrl_zip_path TEXT
        );

        CREATE TABLE IF NOT EXISTS income_statements (
            doc_id TEXT PRIMARY KEY,
            net_sales REAL,
            cost_of_sales REAL,
            gross_profit REAL,
            selling_general_admin REAL,
            operating_income REAL,
            interest_income_noi REAL,
            dividends_income_noi REAL,
            interest_and_dividends_income_noi REAL,
            purchase_discounts_noi REAL,
            rent_income_noi REAL,
            house_rent_income_noi REAL,
            other_noi REAL,
            non_operating_income REAL,
            sales_discounts_noe REAL,
            rent_cost_real_estate_noe REAL,
            other_noe REAL,
            non_operating_expenses REAL,
            ordinary_income REAL,
            gain_on_sales_of_noncurrent_assets_ei REAL,
            extraordinary_income REAL,
            income_before_income_taxes REAL,
            income_taxes_current REAL,
            income_taxes_deferred REAL,
            income_taxes REAL,
            income_before_minority_interests REAL,
            net_income REAL,
            FOREIGN KEY(doc_id) REFERENCES quarterly_reports(doc_id)
        );

        CREATE TABLE IF NOT EXISTS balance_sheets (
            doc_id TEXT PRIMARY KEY,
            cash_and_deposits REAL,
            notes_and_accounts_receivable_trade REAL,
            short_term_investment_securities REAL,
            merchandise REAL,
            property_plant_and_equipment REAL,
            intangible_assets REAL,
            investments_and_other_assets REAL,
            total_assets REAL,
            current_liabilities REAL,
            noncurrent_liabilities REAL,
            total_liabilities REAL,
            shareholders_equity REAL,
            valuation_and_translation_adjustments REAL,
            total_equity REAL,
            FOREIGN KEY(doc_id) REFERENCES quarterly_reports(doc_id)
        );
        "#
    )?;

    let mut current_date = start_date;
    while current_date < end_date {
        let date_str = current_date.format("%Y-%m-%d").to_string();

        match api_client.get_document_list(&date_str).await {
            Ok(api_resp) if api_resp.metadata.status == "200" => {
                for doc in api_resp.results {
                    if let Some(code) = &doc.doc_type_code {
                        if code == "140" || code == "150" {
                            let doc_id = doc.doc_id.clone();

                            let xbrl_zip_path = api_client.download_xbrl(&doc_id, &base_dir)
                                .await
                                .unwrap_or(None);
                            
                            let xbrl_content = if let Some(path) = &xbrl_zip_path {
                                let full_path = base_dir.join(path);
                                extract_xbrl_from_zip(full_path.to_str().unwrap())
                                    .and_then(|c| parse_dynamic_xbrl(&c))
                                    .ok()
                            } else {
                                None
                            };

                            let income_statement = xbrl_content.as_ref().map(|xbrl| extract_income_statement(xbrl));
                            let balance_sheet = xbrl_content.as_ref().map(|xbrl| extract_balance_sheet(xbrl));

                            conn.execute(
                                "INSERT INTO quarterly_reports (
                                    doc_id, date, sec_code, doc_type_code,
                                    submit_date_time, edinet_code, filer_name,
                                    xbrl_zip_path
                                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                                ON CONFLICT(doc_id) DO UPDATE SET
                                    date = ?2,
                                    sec_code = ?3,
                                    doc_type_code = ?4,
                                    submit_date_time = ?5,
                                    edinet_code = ?6,
                                    filer_name = ?7,
                                    xbrl_zip_path = ?8",
                                params![
                                    doc_id,
                                    date_str,
                                    doc.sec_code,
                                    code,
                                    doc.submit_date_time,
                                    doc.edinet_code,
                                    doc.filer_name,
                                    xbrl_zip_path,
                                ],
                            )?;

                            if let Some(stmt) = income_statement {
                                conn.execute(
                                    "INSERT INTO income_statements (
                                        doc_id, net_sales, cost_of_sales, gross_profit,
                                        selling_general_admin, operating_income,
                                        interest_income_noi, dividends_income_noi,
                                        interest_and_dividends_income_noi,
                                        purchase_discounts_noi, rent_income_noi,
                                        house_rent_income_noi, other_noi,
                                        non_operating_income, sales_discounts_noe,
                                        rent_cost_real_estate_noe, other_noe,
                                        non_operating_expenses, ordinary_income,
                                        gain_on_sales_of_noncurrent_assets_ei,
                                        extraordinary_income, income_before_income_taxes,
                                        income_taxes_current, income_taxes_deferred,
                                        income_taxes, income_before_minority_interests,
                                        net_income
                                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                                            ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19,
                                            ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27)
                                    ON CONFLICT(doc_id) DO UPDATE SET
                                        net_sales = ?2,
                                        cost_of_sales = ?3,
                                        gross_profit = ?4,
                                        selling_general_admin = ?5,
                                        operating_income = ?6,
                                        interest_income_noi = ?7,
                                        dividends_income_noi = ?8,
                                        interest_and_dividends_income_noi = ?9,
                                        purchase_discounts_noi = ?10,
                                        rent_income_noi = ?11,
                                        house_rent_income_noi = ?12,
                                        other_noi = ?13,
                                        non_operating_income = ?14,
                                        sales_discounts_noe = ?15,
                                        rent_cost_real_estate_noe = ?16,
                                        other_noe = ?17,
                                        non_operating_expenses = ?18,
                                        ordinary_income = ?19,
                                        gain_on_sales_of_noncurrent_assets_ei = ?20,
                                        extraordinary_income = ?21,
                                        income_before_income_taxes = ?22,
                                        income_taxes_current = ?23,
                                        income_taxes_deferred = ?24,
                                        income_taxes = ?25,
                                        income_before_minority_interests = ?26,
                                        net_income = ?27",
                                    params![
                                        doc_id,
                                        stmt.net_sales,
                                        stmt.cost_of_sales,
                                        stmt.gross_profit,
                                        stmt.selling_general_admin,
                                        stmt.operating_income,
                                        stmt.interest_income_noi,
                                        stmt.dividends_income_noi,
                                        stmt.interest_and_dividends_income_noi,
                                        stmt.purchase_discounts_noi,
                                        stmt.rent_income_noi,
                                        stmt.house_rent_income_noi,
                                        stmt.other_noi,
                                        stmt.non_operating_income,
                                        stmt.sales_discounts_noe,
                                        stmt.rent_cost_real_estate_noe,
                                        stmt.other_noe,
                                        stmt.non_operating_expenses,
                                        stmt.ordinary_income,
                                        stmt.gain_on_sales_of_noncurrent_assets_ei,
                                        stmt.extraordinary_income,
                                        stmt.income_before_income_taxes,
                                        stmt.income_taxes_current,
                                        stmt.income_taxes_deferred,
                                        stmt.income_taxes,
                                        stmt.income_before_minority_interests,
                                        stmt.net_income,
                                    ],
                                )?;
                            }

                            if let Some(sheet) = balance_sheet {
                                conn.execute(
                                    "INSERT INTO balance_sheets (
                                        doc_id, cash_and_deposits,
                                        notes_and_accounts_receivable_trade,
                                        short_term_investment_securities, merchandise,
                                        property_plant_and_equipment, intangible_assets,
                                        investments_and_other_assets, total_assets,
                                        current_liabilities, noncurrent_liabilities,
                                        total_liabilities, shareholders_equity,
                                        valuation_and_translation_adjustments,
                                        total_equity
                                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                                            ?11, ?12, ?13, ?14, ?15)
                                    ON CONFLICT(doc_id) DO UPDATE SET
                                        cash_and_deposits = ?2,
                                        notes_and_accounts_receivable_trade = ?3,
                                        short_term_investment_securities = ?4,
                                        merchandise = ?5,
                                        property_plant_and_equipment = ?6,
                                        intangible_assets = ?7,
                                        investments_and_other_assets = ?8,
                                        total_assets = ?9,
                                        current_liabilities = ?10,
                                        noncurrent_liabilities = ?11,
                                        total_liabilities = ?12,
                                        shareholders_equity = ?13,
                                        valuation_and_translation_adjustments = ?14,
                                        total_equity = ?15",
                                    params![
                                        doc_id,
                                        sheet.assets.cash_and_deposits,
                                        sheet.assets.notes_and_accounts_receivable_trade,
                                        sheet.assets.short_term_investment_securities,
                                        sheet.assets.merchandise,
                                        sheet.assets.property_plant_and_equipment,
                                        sheet.assets.intangible_assets,
                                        sheet.assets.investments_and_other_assets,
                                        sheet.assets.total_assets,
                                        sheet.liabilities.current_liabilities,
                                        sheet.liabilities.noncurrent_liabilities,
                                        sheet.liabilities.total_liabilities,
                                        sheet.equity.shareholders_equity,
                                        sheet.equity.valuation_and_translation_adjustments,
                                        sheet.equity.total_equity,
                                    ],
                                )?;
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error fetching documents for {}: {}", date_str, e),
            _ => (),
        }

        current_date += ChronoDuration::days(1);
        sleep(Duration::from_millis(250)).await;
    }

    Ok(())
}

fn extract_xbrl_from_zip(zip_path: &str) -> Result<String, Box<dyn Error>> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name().ends_with(".xbrl") {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            return Ok(content);
        }
    }

    Err("No .xbrl file found in the zip archive".into())
}

fn parse_dynamic_xbrl(content: &str) -> Result<DynamicXBRLContent, Box<dyn Error>> {
    let mut reader = Reader::from_str(content);
    reader.trim_text(true);

    let mut namespaces = HashMap::new();
    let mut contexts = HashMap::new();
    let mut elements = Vec::new();
    let mut element_stack = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8(e.name().as_ref().to_vec())?;

                let mut attributes = HashMap::new();
                let mut context_ref = None;
                let mut unit_ref = None;

                for attr in e.attributes() {
                    let attr = attr?;
                    let key = String::from_utf8(attr.key.as_ref().to_vec())?;
                    let value = String::from_utf8(attr.value.to_vec())?;

                    match key.as_str() {
                        "contextRef" => context_ref = Some(value),
                        "unitRef" => unit_ref = Some(value),
                        _ => {
                            if key.starts_with("xmlns:") {
                                namespaces.insert(key[6..].to_string(), value.clone());
                            }
                            attributes.insert(key, value);
                        }
                    }
                }

                element_stack.push(XBRLElement {
                    name,
                    value: None,
                    attributes,
                    children: Vec::new(),
                    context_ref,
                    unit_ref,
                });
            },
            Ok(Event::Text(e)) => {
                if let Some(element) = element_stack.last_mut() {
                    element.value = Some(String::from_utf8(e.to_vec())?);
                }
            },
            Ok(Event::End(_)) => {
                if let Some(element) = element_stack.pop() {
                    if element_stack.is_empty() {
                        elements.push(element);
                    } else if let Some(parent) = element_stack.last_mut() {
                        parent.children.push(element);
                    }
                }
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Error parsing XBRL: {:?}", e).into()),
            _ => (),
        }
    }

    Ok(DynamicXBRLContent {
        namespaces,
        contexts,
        elements,
    })
}