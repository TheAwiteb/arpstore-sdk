use machineid_rs::{Encryption, HWIDComponent, IdBuilder};
use sha2::{Digest, Sha256};

/// Errors from the ARPStore API.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An error from the reqwest crate.
    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// An error from the serde_json crate.
    #[error("JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    /// An error from the ARP API.
    #[error("Error: {0}")]
    Api(String),
}

#[allow(clippy::format_collect)]
fn create_signature(
    subscription_key: &str,
    device_hash: &str,
    data: &str,
    timestamp: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(subscription_key.as_bytes());
    hasher.update(device_hash.as_bytes());
    hasher.update(data.as_bytes());
    hasher.update(timestamp.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

fn add_headers(
    request: reqwest::RequestBuilder,
    subscription_key: &str,
) -> reqwest::RequestBuilder {
    request
        .header("X-API-KEY", subscription_key)
        .header("Date", chrono::Utc::now().to_rfc2822())
}

fn add_body(
    request: reqwest::RequestBuilder,
    subscription_key: &str,
    device_hash: &str,
    data: &str,
) -> reqwest::RequestBuilder {
    let timestamp = chrono::Utc::now().timestamp();
    let signature = create_signature(subscription_key, device_hash, data, &timestamp.to_string());
    request.json(&serde_json::json!(
        {
            "device_hash": device_hash,
            "data": data,
            "timestamp": timestamp,
            "signature": signature
        }
    ))
}

/// A client for the ARPStore API.
#[derive(Debug)]
pub struct Client {
    client: reqwest::Client,
    subscription_key: String,
    data: String,
    arp_url: String,
}

impl Client {
    /// Create a new client for the ARPStore API.
    pub fn new(arp_url: impl Into<String>, subscription_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            subscription_key: subscription_key.into(),
            data: String::new(),
            arp_url: arp_url.into(),
        }
    }

    /// Add a data to the subsction, default is empty
    pub fn data(mut self, data: impl Into<String>) -> Self {
        self.data = data.into();
        self
    }

    /// Check if a subscription is valid.
    pub async fn is_valid_subscription(&self, product_code: &str) -> Result<(), Error> {
        let device_hash = IdBuilder::new(Encryption::SHA256)
            .add_component(HWIDComponent::CPUID)
            .add_component(HWIDComponent::SystemID)
            .add_component(HWIDComponent::DriveSerial)
            .build(&self.subscription_key)
            .expect("this should never fail, because we're not using machine name or os name");
        let request_builder = add_body(
            add_headers(
                self.client
                    .post(format!("{}/activation_check", self.arp_url)),
                &self.subscription_key,
            ),
            &self.subscription_key,
            &device_hash,
            &self.data,
        );
        let response = request_builder.send().await?;
        let status = response.status();
        let message = response
            .json::<serde_json::Value>()
            .await?
            .get("message")
            .unwrap()
            .to_string();
        let message = message.trim_matches('"').replace(r#"\n"#, "\n").to_owned();

        if status == reqwest::StatusCode::OK {
            if message.ends_with(product_code) {
                Ok(())
            } else {
                Err(Error::Api(
                    "The subscription is not for this product!".to_string(),
                ))
            }
        } else {
            Err(Error::Api(message))
        }
    }
}
