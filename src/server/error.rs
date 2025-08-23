#![allow(dead_code)]

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
    InvalidToken,
    Overloaded,
    GameDoesNotExist,
    InvalidRequest,
}

impl ErStatus {
    pub fn json(&self) -> String {
        match self {
            ErStatus::Pending => Er::status("pending"),
            ErStatus::InvalidToken => Er::status("invalid_token"),
            ErStatus::Overloaded => Er::status("overloaded"),
            ErStatus::GameDoesNotExist => Er::status("game_does_not_exist"),
            ErStatus::InvalidRequest => Er::status("invalid_request"),
        }
    }
}
