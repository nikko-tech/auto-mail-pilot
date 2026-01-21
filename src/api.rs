use crate::models::{Template, RecipientData, Signature, LinkingData};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone)]
pub struct GasClient {
    client: Client,
    url: String,
}

#[derive(Deserialize)]
struct GetTemplatesResponse {
    templates: Option<Vec<Template>>,
    error: Option<String>,
}

#[derive(Deserialize)]
pub struct GetRecipientsResponse {
    pub recipients: Vec<RecipientData>,
    // error: Option<String>, // Removed error field as per instruction's implied change
}

#[derive(Deserialize)]
pub struct GetSignaturesResponse {
    pub signatures: Vec<Signature>,
}

#[derive(Deserialize)]
pub struct GetLinkingsResponse {
    pub linkings: Vec<LinkingData>,
}

#[derive(Deserialize)]
struct PostResponse {
    success: bool,
    error: Option<String>,
}

#[derive(Serialize)]
struct BatchMailItem<'a> {
    to: &'a str,
    subject: &'a str,
    body: &'a str,
}

impl GasClient {
    pub fn new(url: String) -> Self {
        Self {
            client: Client::new(),
            url,
        }
    }

    pub fn get_templates(&self) -> Result<Vec<Template>, String> {
        let mut base_url = self.url.trim().to_string();
        if base_url.is_empty() {
            return Err("GAS URL is not set".to_string());
        }

        if let Some(pos) = base_url.find('?') {
            base_url.truncate(pos);
        }

        let response = self.client.get(&base_url)
            .query(&[("action", "getTemplates")])
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        let parsed: GetTemplatesResponse = response.json().map_err(|e| format!("JSON parse error: {}", e))?;

        if let Some(error) = parsed.error {
            return Err(error);
        }

        Ok(parsed.templates.unwrap_or_default())
    }

    pub fn get_recipients(&self) -> Result<Vec<RecipientData>, String> {
        let mut base_url = self.url.trim().to_string();
        if base_url.is_empty() {
            return Err("GAS URL is not set".to_string());
        }

        if let Some(pos) = base_url.find('?') {
            base_url.truncate(pos);
        }

        let response = self.client.get(&base_url)
            .query(&[("action", "getRecipients")])
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        let parsed: GetRecipientsResponse = response.json().map_err(|e| format!("JSON parse error: {}", e))?;

        // The original code had error handling here, but the instruction implies removing the error field from GetRecipientsResponse.
        // If the API can still return an error, this logic needs to be re-evaluated.
        // For now, assuming the new GetRecipientsResponse structure means no direct error field.
        Ok(parsed.recipients)
    }

    pub fn get_signatures(&self) -> Result<Vec<Signature>, String> {
        let mut base_url = self.url.trim().to_string();
        if base_url.is_empty() {
            return Err("GAS URL is not set".to_string());
        }

        if let Some(pos) = base_url.find('?') {
            base_url.truncate(pos);
        }

        let url = format!("{}?action=getSignatures", base_url);
        let response = self.client.get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        let parsed: GetSignaturesResponse = response.json().map_err(|e| format!("JSON parse error: {}", e))?;
        Ok(parsed.signatures)
    }

    pub fn get_linkings(&self) -> Result<Vec<LinkingData>, String> {
        let mut base_url = self.url.trim().to_string();
        if base_url.is_empty() {
            return Err("GAS URL is not set".to_string());
        }

        if let Some(pos) = base_url.find('?') {
            base_url.truncate(pos);
        }

        let url = format!("{}?action=getLinkings", base_url);
        let response = self.client.get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        let parsed: GetLinkingsResponse = response.json().map_err(|e| format!("JSON parse error: {}", e))?;
        Ok(parsed.linkings)
    }

    pub fn send_mail(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        let mut base_url = self.url.trim().to_string();
        if base_url.is_empty() {
             return Err("GAS URL is not set".to_string());
        }

        if let Some(pos) = base_url.find('?') {
            base_url.truncate(pos);
        }

        let payload = json!({
            "action": "sendMail",
            "to": to,
            "subject": subject,
            "body": body,
        });

        let response = self.client.post(&base_url)
            .json(&payload)
            .send()
             .map_err(|e| format!("Request failed: {}", e))?;

        let parsed: PostResponse = response.json().map_err(|e| format!("JSON parse error: {}", e))?;

        if !parsed.success {
            return Err(parsed.error.unwrap_or("Unknown error".to_string()));
        }
        Ok(())
    }

    pub fn send_batch_mail(&self, items: Vec<(&str, &str, &str)>) -> Result<(), String> {
        let mut base_url = self.url.trim().to_string();
        if base_url.is_empty() {
            return Err("GAS URL is not set".to_string());
        }

        if let Some(pos) = base_url.find('?') {
            base_url.truncate(pos);
        }

        let emails: Vec<BatchMailItem> = items.into_iter()
            .map(|(to, sub, body)| BatchMailItem { to, subject: sub, body })
            .collect();

        let payload = json!({
            "action": "sendBatchMail",
            "emails": emails,
        });

        let response = self.client.post(&base_url)
            .json(&payload)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        let parsed: PostResponse = response.json().map_err(|e| format!("JSON parse error: {}", e))?;

        if !parsed.success {
            return Err(parsed.error.unwrap_or("Unknown error".to_string()));
        }
        Ok(())
    }
}
