#![allow(dead_code)]


use std::sync::Arc;
use crate::token::*;
use dashmap::DashMap;
use dashmap::mapref::one::RefMut;

pub fn sanitize_question(question: &String) -> Option<String> {
    if question.len() > 120 {
        return None;
    }
    Some(question.clone())
}

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

pub struct Record {
    questions: Question,
    answers: Option<Answer>,
}

#[derive(Default)]
pub struct GameState {
    subject: String,
    records: Vec<Record>,
    pending_question: Option<Question>,
    versions: u32,
}

pub struct GameManager {
    game_states: Arc<DashMap<Token, GameState>>,
}


impl Record {
    fn new(question: String) -> Self {
        Record {
            questions: Question {text: question},
            answers: None,
        }
    }

    fn answer(&mut self, verdict: Verdict, comment: String) {
        self.answers = Some(Answer{ verdict, comment});
    }
}


impl GameState {
    fn touch(&mut self) {
        self.versions += 1;
    }

    pub fn get_version(&self) -> u32 {
        self.versions
    }

    pub fn set_pending_question(&mut self, question: &String) -> bool {
        if self.pending_question.is_some() {
            return false;
        }
        self.pending_question = Some(Question{text: question.clone()});
        self.touch();
        true
    }

    pub fn add_record(&mut self, record: Record) {
        self.records.push(record);
        self.touch();
    }
}

impl GameManager {
    pub fn new() -> Self {
        GameManager {
            game_states: Arc::new(DashMap::new()),
        }
    }

    pub fn get_game(&self, token: &Token) -> Option<RefMut<Token, GameState>> {
        let rv = self.game_states.get_mut(token);
        rv
    }


    pub fn new_game(&self) -> Token {
        let token = Token::new(TokenType::Game);
        self.game_states.insert(token.clone(), GameState::default());
        token
    }
}

