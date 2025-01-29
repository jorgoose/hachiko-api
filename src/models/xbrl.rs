use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use serde_json::Value as JsonValue;

#[derive(Serialize, Deserialize, Debug)]
pub struct XBRLElement {
    pub name: String,
    pub value: Option<String>,
    pub attributes: HashMap<String, String>,
    pub children: Vec<XBRLElement>,
    pub context_ref: Option<String>,
    pub unit_ref: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DynamicXBRLContent {
    pub namespaces: HashMap<String, String>,
    pub contexts: HashMap<String, JsonValue>,
    pub elements: Vec<XBRLElement>,
}
