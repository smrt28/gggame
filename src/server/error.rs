use serde_json::json;

pub struct Er{}

impl Er{
    fn error(message: &str) -> String {
        json!({
            "status": "error",
            "message": message
        }).to_string()
    }

    fn status(status: &str) -> String {
        json!({
            "status": status,
        }).to_string()
    }
}

pub enum ErStatus {
    Pending,
    Error(String),
    InvalidToken,
    Overloaded,
}

impl ErStatus {
    pub fn json(&self) -> String {
        match self {
            ErStatus::Pending => Er::status("pending"),
            ErStatus::InvalidToken => Er::status("invalid_token"),
            ErStatus::Error(text) => Er::error(text.as_str()),
            ErStatus::Overloaded => Er::status("overloaded")
        }
    }

    pub fn error(message: &str) -> ErStatus {
        ErStatus::Error(message.to_string())
    }
}
