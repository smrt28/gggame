#![allow(dead_code)]

use rand::Rng;
use anyhow::{Context, Result};

pub const TOKEN_LENGTH: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TokenType {
    Answer,
    Game,
}

impl TokenType {
    fn leading_byte(&self) -> u8 {
        match self {
            TokenType::Answer => 'a' as u8,
            TokenType::Game => 'g' as u8,
        }
    }
    fn get_token_type(token: &Token) -> Option<TokenType> {
        match token.token[0] as char {
            'a' => Some(TokenType::Answer),
            'g' => Some(TokenType::Game),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct Token {
    token: [u8; TOKEN_LENGTH],
}

impl Token {
    fn random_bytes(leading_letter: u8) -> [u8; TOKEN_LENGTH] {
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::rng();
        let mut buf = [0u8; TOKEN_LENGTH];
        buf[0] = leading_letter;
        for b in &mut buf[1..] {
            *b = CHARSET[rng.random_range(0..CHARSET.len())];
        }
        buf
    }

    pub fn from_stringr(token_str: &str) -> Result<Self> {
        let bytes = token_str.bytes();
        if bytes.len() == TOKEN_LENGTH {
            let mut token = [0u8; TOKEN_LENGTH];
            for (i, b) in bytes.enumerate() {
                token[i] = b;
            }
            TokenType::get_token_type(&Self { token }).context("Not a token")?;
            return Ok(Self { token });
        }
        Err(anyhow::anyhow!("Not a token"))
    }

    pub fn get_token_type(&self) -> TokenType {
        TokenType::get_token_type(self).unwrap()
    }

    pub fn new(token_type: TokenType) -> Self {
        Self {
            token: Self::random_bytes(token_type.leading_byte())
        }
    }

    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.token).to_string()
    }

    pub fn to_str(&self) -> &str {
        std::str::from_utf8(&self.token).unwrap()
    }
}