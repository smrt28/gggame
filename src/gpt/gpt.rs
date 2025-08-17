

#![allow(unused_attributes)]
#![allow(unused_imports)]
#![allow(dead_code)]

use crate::string_enum;
use anyhow::{Context, Result};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

string_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Model {
        //Gpt4o => "gpt-4o-mini",
        Gpt5  => "gpt-5",
        Gpt5Mini => "gpt-5-mini",
        Gpt5Nano => "gpt-5-nano",
    }
}

pub struct Answer {
    response: Response,
    json: Value,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "output_text")]
    OutputText { text: String },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum OutputItem {
    #[serde(rename = "message")]
    Message {
        #[serde(default)]
        content: Vec<ContentPart>
    },
    #[serde(other)] // ignore "reasoning" or anything else
    Other,
}


#[derive(Deserialize)]
pub struct Response {
    #[serde(default)]
    output: Vec<OutputItem>,


    //output_text: Option<String>, // sometimes provided by API
}

impl Response {
    fn first_output_text_typed(&self) -> Option<&str> {
        for item in &self.output {
            if let OutputItem::Message { content } = item {
                for part in content {
                    if let ContentPart::OutputText { text } = part {
                        return Some(text.as_str());
                    }
                }
            }
        }
        None
    }
}


impl Answer {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            json: serde_json::from_slice(&bytes).context("JSON parse failed")?,
            response: serde_json::from_slice(&bytes)?,
        })
    }

    pub fn to_string(&self) -> Option<String> {
        self.response.first_output_text_typed().map(|s| s.to_string())
    }

    pub fn dump(&self) {
        if let Ok(s) = serde_json::to_string_pretty(&self.json) {
            println!("{}", s);
        }
    }
}

pub struct GptClient {
    client: reqwest::Client,
    key: Option<String>,
}

string_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Verbosity {
        Low => "low",
        Medium => "medium",
        High => "high",
    }
}

pub struct QuestionParams {
    verbosity: Verbosity,
    model: Model,
    instructions: Option<String>,
    max_output_tokens: Option<i32>,
    temperature: Option<f32>,
}

impl QuestionParams {
    pub fn default() -> Self {
        Self {
            verbosity: Verbosity::Medium,
            model: Model::Gpt5Nano,
            instructions: None,
            max_output_tokens: None,
            temperature: None,
        }
    }

    #[allow(dead_code)]
    pub fn set_model(&mut self, model: Model) {
        self.model = model;
    }

    #[allow(dead_code)]
    pub fn set_instructions<S: AsRef<str>>(&mut self, instructions: S) {
        let s = instructions.as_ref().trim();
        if s.len() <= 1 { return };
        self.instructions = Some(s.to_owned());
    }

    #[allow(dead_code)]
    pub fn set_max_output_tokens(&mut self, max_output_tokens: i32) {
        self.max_output_tokens = Some(max_output_tokens);
    }

    #[allow(dead_code)]
    pub fn set_temperature(&mut self, temperature: f32) {
        self.temperature = Some(temperature);
    }
}

#[derive(Serialize)]
struct RequestBody<'a> {
    model: String,
    input: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<&'a str>,
    text: serde_json::Value,
    max_output_tokens: Option<i32>,
    temperature: Option<f32>,
}


 impl GptClient {
    pub fn new() -> Self {
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

    pub fn read_gpt_key_from_file(&mut self, path_opt: Option<String>) -> Result<()> {
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


    pub async fn ask(&self, question: &str, params: &QuestionParams) -> Result<Answer> {
        let body = RequestBody {
            model: params.model.to_string(),
            input: question,
            temperature: params.temperature,
            instructions: params.instructions.as_deref(),
            max_output_tokens: params.max_output_tokens,
            text: json!({ "verbosity": params.verbosity.to_string() }),
        };

        let body = serde_json::to_value(&body)?;

        let resp = self.client
            .post("https://api.openai.com/v1/responses")
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, format!("Bearer {}", self.get_key()?))
            .json(&body)
            .send()
            .await
            .context("HTTP request failed")?;

        let status = resp.status();
        let bytes = resp.bytes().await.context("reading body failed")?;

        if !status.is_success() {
            let text = String::from_utf8_lossy(&bytes);
            anyhow::bail!("OpenAI error {}: {}", status, text);
        }

        Ok(Answer::from_bytes(&bytes)?)
    }
}

