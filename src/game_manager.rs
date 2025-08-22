#![allow(dead_code)]

use crate::token::*;
use dashmap::DashMap;

enum Verdict {
    Yes, No, Unable
}


struct Question {
    text: String,
}

struct Answer {
    verdict: Verdict,
    comment: String,
}

struct Record {
    questions: String,
    answers: Option<Answer>,
}

struct GameState {
    subject: String,
    records: Vec<Record>,
    pending_question: Option<String>,
}

struct GameManager {
    game_states: DashMap<String, GameState>,
}


impl Record {
    fn new(question: String) -> Self {
        Record {
            questions: question,
            answers: None,
        }
    }

    fn answer(&mut self, verdict: Verdict, comment: String) {
        self.answers = Some(Answer{ verdict, comment});
    }
}


impl GameManager {
    pub fn new() -> Self {
        GameManager {
            game_states: DashMap::new(),
        }
    }
    
    
}

