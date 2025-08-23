#![allow(dead_code)]
use std::time::Duration;
use rand::Rng;
use std::collections::VecDeque;
use std::iter;
use std::sync::Arc;
use linked_hash_map::LinkedHashMap;
use tokio::sync::Notify;
use crate::utinls::*;


#[derive(Clone)]
pub enum SlotState {
    Pending,
    Content(String),
}


#[derive(Clone)]
pub struct Slot {
    pub(crate) notify: Arc<Notify>,
    state: SlotState,
}

impl Slot {
    fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
            state: SlotState::Pending,
        }
    }
}

#[derive(Default)]
pub struct AnswerCache {
    map: LinkedHashMap<String, Slot>,
    limit: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnswerCacheEntry {
    Text(String),
    Pending,
    None,
}

impl AnswerCache {
    pub fn new() -> Self {
        let mut res = Self::default();
        res.limit = 2048;
        res
    }

    pub fn reserve_token(&mut self) -> String {
        let token = Self::generate_token();
        self.map.insert(token.clone(), Slot::new());

        while self.map.len() > self.limit {
            self.map.pop_front(); // removes the oldest
        }

        token
    }

    pub fn insert(&mut self, token: &str, text: &str) -> bool {
        if let Some(slot) = self.map.get_mut(token) {
            slot.state = SlotState::Content(text.to_owned());
            return true;
        }
        false
    }

    pub fn get(&self, token: &str) -> AnswerCacheEntry {
        match self.map.get(token) {
            Some(slot) => {
                match &slot.state {
                    SlotState::Content(text) => AnswerCacheEntry::Text(text.to_string()),
                    SlotState::Pending => AnswerCacheEntry::Pending,
                }
            }
            None => AnswerCacheEntry::None,
        }
    }

    pub fn snapshot(&self, token: &str) -> Option<Slot> {
        self.map.get(token).cloned()
    }

    pub fn generate_token() -> String {
         token_generator::generate_token()
    }
}
