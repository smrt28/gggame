use anyhow::{Context, Result};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};


struct GptClient {
    client: reqwest::Client,
    key: Option<String>,
}

impl GptClient {
    fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            key: None,
        }
    }


    fn get_key(&self) -> anyhow::Result<&str> {
        self.key
            .as_ref()
            .map(|s| s.as_str())
            .ok_or_else(|| anyhow::anyhow!("key not set"))
    }

    fn read_gpt_key_from_file(&mut self, path_opt: Option<String>) -> Result<()> {
        let path: PathBuf = match path_opt {
            Some(p) => PathBuf::from(p),
            None => {
                let home = env::var("HOME")
                    .context("HOME is not set; pass a path or set HOME")?;
                PathBuf::from(home).join(".gpt.key")
            }
        };

        let contents = fs::read_to_string(&path)
            .with_context(|| format!("reading key file at {}", path.display()))?;

        let key = contents.trim().to_string();
        if key.is_empty() {
            anyhow::bail!("key file {} is empty/whitespace", path.display());
        }

        self.key = Some(key.clone());
        Ok(())
    }
}


#[tokio::main]
async fn main() -> Result<()> {

    let mut cli = GptClient::new();
    cli.read_gpt_key_from_file(None)?;



    let home_path = env::var("HOME").expect("HOME env var not set");
    let key_path = PathBuf::from(home_path).join(".gpt.key");
    let key = fs::read_to_string(&key_path)
        .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", key_path, e))
        .trim()
        .to_string();

    let body = serde_json::json!({
        "model": "gpt-4o-mini",
        "input": "how are you?"
    });

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.openai.com/v1/responses")
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", key))
        .json(&body)
        .send()
        .await
        .context("HTTP request failed")?;

    // 👇 borrow-of-moved-value fix: copy status *before* consuming the body
    let status = resp.status();
    let bytes = resp.bytes().await.context("reading body failed")?;

    if !status.is_success() {
        // Body was already read; use it here.
        let text = String::from_utf8_lossy(&bytes);
        anyhow::bail!("OpenAI error {}: {}", status, text);
    }

    // Parse once; extract output_text if present, else dump the JSON.
    let v: Value = serde_json::from_slice(&bytes).context("JSON parse failed")?;
    if let Some(text) = v.get("output_text").and_then(|x| x.as_str()) {
        println!("{}", text.trim());
    } else {
        println!("{}", serde_json::to_string_pretty(&v)?);
    }



    Ok(())
}