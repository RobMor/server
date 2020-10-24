use anyhow::{anyhow, Context, Result};
use num_bigint::BigInt;
use openssl::sha::Sha1;
use reqwest::Client;
use serde_json::Value;
use uuid::Uuid;

pub async fn authenticate(username: &str, shared_secret: &[u8], public_key: &[u8]) -> Result<Uuid> {
    let mut hasher = Sha1::new();
    hasher.update(b"");
    hasher.update(shared_secret);
    hasher.update(public_key);
    let hash = hasher.finish();

    let hex = format!("{:x}", BigInt::from_signed_bytes_be(&hash));

    let client = Client::new();
    let result = client
        .get("https://sessionserver.mojang.com/session/minecraft/hasJoined")
        .query(&[("username", username), ("serverId", &hex)])
        .send()
        .await
        .context("Failed to query Mojang API")?;

    let data = result.json::<Value>().await?;

    Ok(Uuid::parse_str(
        data["id"]
            .as_str()
            .ok_or_else(|| anyhow!("Malformed response JSON"))?,
    )?)
}
