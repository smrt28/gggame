use serde_json::json;

pub struct Er{}

enum ErStatus {
    Pending,
    Error(String),
    InvalidToken
}

impl Er{
    pub fn error(message: &str) -> String {
        json!({
            "status": "error",
            "message": message
        }).to_string()
    }

    pub fn status(status: &str) -> String {
        json!({
            "status": status,
        }).to_string()
    }
}
