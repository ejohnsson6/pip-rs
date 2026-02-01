use eyre::Context;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const CLOUDFLARE_API_URL: &'static str = "https://api.cloudflare.com/client/v4";

pub struct CloudflareClient {
    auth_token: String,
    http_client: Client,
    /// Don't do any writes, only read operations.
    mock: bool,
}

impl CloudflareClient {
    pub fn new(auth_token: String, mock: bool) -> Self {
        Self {
            auth_token,
            http_client: reqwest::Client::new(),
            mock,
        }
    }

    pub async fn list_dns_records(&self, zone_id: &str) -> eyre::Result<Vec<RecordResponse>> {
        let endpoint = format!("{CLOUDFLARE_API_URL}/zones/{zone_id}/dns_records");
        let response: CloudflareResponse<Vec<RecordResponse>> = self
            .http_client
            .get(endpoint)
            .bearer_auth(&self.auth_token)
            .send()
            .await
            .wrap_err("Failed to send list dns records request")?
            .json()
            .await
            .wrap_err("Failed to deserialize list dns records response")?;

        if !response.success {
            eyre::bail!(
                "Got error respones from cloudflare: \nErrors: {:#?}\nMessages: {:#?}",
                response.errors,
                response.messages
            );
        }

        let Some(result) = response.result else {
            eyre::bail!("Got no response (?) from cloudflare, response: {response:#?}");
        };

        return Ok(result);
    }

    pub async fn overwrite_dns_record(
        &self,
        zone_id: &str,
        dns_record_id: &str,
        record: RecordRequest,
    ) -> eyre::Result<()> {
        if self.mock {
            println!(
                "[MOCK]: Was asked to overwrite dns record {dns_record_id} on zone {zone_id} to record {record:#?}"
            );
            return Ok(());
        }

        let endpoint = format!("{CLOUDFLARE_API_URL}/zones/{zone_id}/dns_records/{dns_record_id}");
        let response: CloudflareResponse<RecordResponse> = self
            .http_client
            .put(endpoint)
            .bearer_auth(&self.auth_token)
            .send()
            .await
            .wrap_err("Failed to send overwrite DNS record request")?
            .json()
            .await
            .wrap_err("Failed to deserialize cloudflare overwrite DNS record response")?;

        if !response.success || response.result.is_none() {
            eyre::bail!("Got error response from cloudflare {response:#?}");
        }

        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct RecordRequest {
    name: String,
    ttl: usize,
    #[serde(rename = "type")]
    record_type: RecordType,
    comment: Option<String>,
    content: Option<String>,
    options: Option<RecordRequestSettings>,
    // Missing TAGS.
}

impl RecordRequest {
    pub fn response_with_content(
        record_response: RecordResponse,
        new_content: Option<String>,
    ) -> Self {
        Self {
            name: record_response.name,
            ttl: record_response.ttl,
            record_type: record_response.record_type,
            comment: record_response.comment,
            content: new_content,
            options: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RecordRequestSettings {
    ipv4_only: bool,
    ipv6_only: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RecordResponse {
    pub name: String,
    pub id: String,
    pub ttl: usize,
    #[serde(rename = "type")]
    pub record_type: RecordType,
    pub content: Option<String>,
    pub comment: Option<String>,
    pub proxied: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecordType {
    A,
    AAAA,
    CAA,
    CERT,
    CNAME,
    DNSKEY,
    DS,
    HTTPS,
    LOC,
    MX,
    NAPTR,
    NS,
    OPENPGPKEY,
    PTR,
    SMIMEA,
    SRV,
    SSHFP,
    SVCB,
    TLSA,
    TXT,
    URI,
}

#[derive(Debug, Deserialize)]
pub struct CloudflareResponse<T> {
    pub errors: Vec<CFError>,
    pub messages: Vec<CFError>,
    pub success: bool,
    pub result: Option<T>,
    pub result_info: Option<ResultInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ResultInfo {
    pub count: usize,
    pub page: usize,
    pub per_page: usize,
    pub total_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct CFError {
    pub code: usize,
    pub message: String,
    pub documentation_url: Option<String>,
    pub source: Option<CFSource>,
}

#[derive(Debug, Deserialize)]
pub struct CFSource {
    pub pointer: Option<String>,
}
