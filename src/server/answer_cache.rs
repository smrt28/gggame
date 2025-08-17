use rand::Rng;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::iter;
use std::sync::Arc;
use linked_hash_map::LinkedHashMap;
use tokio::sync::Notify;

fn generate(len: usize) -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    let one_char = || CHARSET[rng.random_range(0..CHARSET.len())] as char;
    iter::repeat_with(one_char).take(len).collect()
}

#[derive(Clone)]
pub struct Slot {
    value: Option<String>,
    pub(crate) notify: Arc<Notify>,
}

impl Slot {
    fn new() -> Self {
        Self {
            value: None,
            notify: Arc::new(Notify::new()),
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
            slot.value = Some(text.to_owned());
            return true;
        }
        false
    }

    pub fn get(&self, token: &str) -> AnswerCacheEntry {
        match self.map.get(token) {
            Some(slot) => {
                match &slot.value {
                    Some(text) => AnswerCacheEntry::Text(text.to_string()),
                    None => AnswerCacheEntry::Pending,
                }
            }
            None => AnswerCacheEntry::None,
        }
    }

    pub fn snapshot(&self, token: &str) -> Option<Slot> {
        self.map.get(token).cloned()
    }

    pub fn generate_token() -> String {
        format!("t_{}", generate(32))
    }
}
