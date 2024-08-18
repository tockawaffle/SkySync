use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;

/// Represents the type of DNS record.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum DnsType {
    A,
    AAAA,
    CNAME,
    HTTPS,
    TXT,
    SRV,
}

/// Contains information about the result of a DNS query.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ResultInfo {
    pub page: i64,
    pub per_page: i64,
    pub count: i64,
    pub total_count: i64,
    pub total_pages: i64,
}

/// Metadata associated with a DNS record.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Meta {
    pub auto_added: bool,
    pub managed_by_apps: bool,
    pub managed_by_argo_tunnel: bool,
}

/// Represents a DNS record.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Struct {
    pub id: String,
    pub zone_id: String,
    pub zone_name: String,
    pub name: String,
    pub r#type: String,
    pub content: String,
    pub proxiable: bool,
    pub proxied: bool,
    pub ttl: i64,
    pub meta: Meta,
    pub comment: Option<String>,
    pub tags: Vec<String>,
    pub created_on: String,
    pub modified_on: String,
    pub comment_modified_on: Option<String>,
}

/// Root structure for the DNS records response.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Root {
    pub result: Vec<Struct>,
    pub success: bool,
    pub errors: Vec<String>,
    pub messages: Vec<String>,
    pub result_info: ResultInfo,
}

/// Fetches DNS records from Cloudflare.
///
/// # Arguments
/// * `dns_type` - An optional `DnsType` to filter the DNS records.
///
/// # Returns
/// A `Root` structure containing the DNS records.
pub(crate) async fn dns_records(dns_type: Option<DnsType>) -> std::result::Result<Root, Box<dyn Error>> {
    dotenv().ok();
    let dns_type = match dns_type {
        Some(dns_type) => match dns_type {
            DnsType::A => "A",
            DnsType::AAAA => "AAAA",
            DnsType::CNAME => "CNAME",
            DnsType::HTTPS => "HTTPS",
            DnsType::TXT => "TXT",
            DnsType::SRV => "SRV"
        },
        None => ""
    };

    let cf_zone_id = env::var("CF_ZONE_ID")?;
    let uri = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records?type={}", cf_zone_id, dns_type);

    let cf_api_key = env::var("CF_API_KEY")?;
    let cf_email = env::var("CF_EMAIL")?;

    let client = reqwest::Client::new();
    let response = client.get(&uri)
        .header("X-Auth-Email", cf_email)
        .header("X-Auth-Key", cf_api_key)
        .send()
        .await?;

    let data = response.text().await?;
    let root: Root = serde_json::from_str(&data)?;
    Ok(root)
}

/// Metadata associated with a DNS record (alternative structure).
#[derive(Serialize, Deserialize)]
pub(crate) struct Meta1 {
    pub auto_added: bool,
    pub managed_by_apps: bool,
    pub managed_by_argo_tunnel: bool,
}

/// Represents a DNS record (alternative structure).
#[derive(Serialize, Deserialize)]
pub(crate) struct Result {
    pub id: String,
    pub zone_id: String,
    pub zone_name: String,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub content: String,
    pub proxiable: bool,
    pub proxied: bool,
    pub ttl: i64,
    pub meta: Meta1,
    pub comment: Option<String>,
    pub tags: Vec<String>,
    pub created_on: String,
    pub modified_on: String,
}

/// Response structure for updating DNS records.
#[derive(Serialize, Deserialize)]
pub(crate) struct UpdateResponse {
    pub result: Result,
    pub success: bool,
    pub errors: Vec<String>,
    pub messages: Vec<String>,
}

/// Updates a DNS record in Cloudflare.
///
/// # Arguments
/// * `id` - The ID of the DNS record to update.
/// * `dns_type` - The type of DNS record.
/// * `name` - The name of the DNS record.
/// * `content` - The content of the DNS record.
/// * `ttl` - The TTL (Time To Live) of the DNS record.
/// * `proxied` - Whether the DNS record is proxied.
///
/// # Returns
/// An `UpdateResponse` structure containing the result of the update operation.
pub(crate) async fn update_dns_records(
    id: &str,
    dns_type: DnsType,
    name: &str,
    content: &str,
    ttl: i64,
    proxied: bool,
) -> UpdateResponse {
    dotenv().ok();
    let uri = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}", env::var("CF_ZONE_ID").expect("Expected a Cloudflare zone id in the environment"), id);

    let cf_api_key = env::var("CF_API_KEY").expect("Expected a Cloudflare API key in the environment");
    let cf_email = env::var("CF_EMAIL").expect("Expected a Cloudflare email in the environment");

    let client = reqwest::Client::new();
    let response = client.put(&uri)
        .header("X-Auth-Email", cf_email)
        .header("X-Auth-Key", cf_api_key)
        .json(&serde_json::json!({
            "type": dns_type,
            "name": name,
            "content": content,
            "ttl": ttl,
            "proxied": proxied
        }))
        .send()
        .await
        .expect("Failed to send request");

    let data = response.text().await.unwrap();
    let root: UpdateResponse = serde_json::from_str(&data).unwrap();
    root
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the `dns_records` function.
    #[tokio::test]
    async fn test_dns_records() {
        let resp = dns_records(Some(DnsType::A)).await.unwrap();
        println!("{:?}", resp);
        assert_eq!(resp.success, true);
    }

    /// Tests the `update_dns_records` function.
    #[tokio::test]
    async fn test_update_dns_records() {
        let dns_name = match dns_records(None).await.unwrap() {
            // Filter by name
            Root { result, .. } => {
                result.into_iter().find(|x| x.name == "DOMAIN_NAME").unwrap_or_else(|| panic!("Failed to find DNS record"))
            }
        };

        let req = update_dns_records(
            &dns_name.id,
            DnsType::A,
            "DOMAIN_NAME",
            "192.168.15.112",
            dns_name.ttl,
            false,
        ).await;

        assert_eq!(req.success, true);
    }
}