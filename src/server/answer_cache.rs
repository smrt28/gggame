use rand::Rng;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::iter;
use linked_hash_map::LinkedHashMap;

fn generate(len: usize) -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    let one_char = || CHARSET[rng.random_range(0..CHARSET.len())] as char;
    iter::repeat_with(one_char).take(len).collect()
}

#[derive(Default)]
pub struct AnswerCache {
    map: LinkedHashMap<String, Option<String>>,
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
        self.map.insert(token.clone(), None);

        while self.map.len() > self.limit {
            self.map.pop_front(); // removes the oldest
        }

        token
    }

    pub fn insert(&mut self, token: &str, text: &str) -> bool {
        if let Some(slot) = self.map.get_mut(token) {
            *slot = Some(text.to_owned());
            return true;
        }
        false
    }

    pub fn get(&self, token: &str) -> AnswerCacheEntry {
        match self.map.get(token) {
            Some(Some(text)) => AnswerCacheEntry::Text(text.clone()),
            Some(None) => AnswerCacheEntry::Pending,
            None => AnswerCacheEntry::None,
        }
    }

    pub fn generate_token() -> String {
        format!("t-{}", generate(24))
    }
}
