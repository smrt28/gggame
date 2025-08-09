#![allow(unused_attributes)]
#![allow(unused_imports)]

mod string_enum;


use anyhow::{Context, Result};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::string_enum::string_enum;

string_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Model {
        Gpt4o => "gpt-4o-mini",
        Gpt5  => "gpt-5",
        Gpt5Mini  => "gpt-5-mini",
    }
}







struct Answer {
    response: Response,
    json: Value,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ContentPart {
    #[serde(rename = "output_text")]
    OutputText { text: String },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum OutputItem {
    #[serde(rename = "message")]
    Message {
        #[serde(default)]
        content: Vec<ContentPart>
    },
    #[serde(other)] // ignore "reasoning" or anything else
    Other,
}


#[derive(Deserialize)]
struct Response {
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

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            json: serde_json::from_slice(&bytes).context("JSON parse failed")?,
            response: serde_json::from_slice(&bytes)?,
        })
    }

    fn to_string(&self) -> Option<String> {
        self.response.first_output_text_typed().map(|s| s.to_string())
    }

    fn dump(&self) {
        if let Ok(s) = serde_json::to_string_pretty(&self.json) {
            println!("{}", s);
        }
    }
}







struct GptClient {
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

struct QuestionParams {
    verbosity: Verbosity,
    model: Model,
    instructions: Option<String>,
}

impl QuestionParams {
    pub fn default() -> Self {
        Self {
            verbosity: Verbosity::Medium,
            model: Model::Gpt5Mini,
            instructions: None,
        }
    }

    #[allow(dead_code)]
    pub fn set_model(&mut self, model: Model) {
        self.model = model;
    }

    pub fn set_instructions<S: AsRef<str>>(&mut self, instructions: S) {
        let s = instructions.as_ref().trim();
        if s.len() <= 1 { return };
        self.instructions = Some(s.to_owned());
    }
}


#[derive(Serialize)]
struct RequestBody<'a> {
    model: String,
    input: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<&'a str>,
    text: serde_json::Value,
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


    async fn ask(&self, question: &str, params: &QuestionParams) -> Result<Answer> {
        let body = RequestBody {
            model: params.model.to_string(),
            input: question,
            instructions: params.instructions.as_deref(),
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


#[tokio::main]
async fn main() -> Result<()> {

    let mut cli = GptClient::new();
    let mut params = QuestionParams::default();
    params.set_instructions("Short minimalistic answer to the question. 1–2 words unless the correct name naturally requires more. No punctuation, no extra explanation.");


    cli.read_gpt_key_from_file(None)?;
    //let answer = cli.ask("Where is Prague located?", &params).await?;
    let answer = cli.ask("In order to play the game 'guess the animal', Choose the animal by random you are going to be and tell me the animal.", &params).await?;
    let res = answer.to_string().unwrap_or(String::new());
    println!("{}", res);

    //answer.dump();
    Ok(())
}