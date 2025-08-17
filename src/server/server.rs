
#![allow(dead_code)]

use axum::{
    extract::ConnectInfo,
    routing::get,
    Router,
    extract::State,
    extract::Path,
};


use anyhow::{Context, Error, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use axum::response::Html;
use tokio::{net::TcpListener, sync::Mutex};
use std::sync::Mutex as StdMutex;
use clap::builder::Str;
use serde_json::json;
use crate::{gpt, GptClientFactory};
use crate::server::client_pool::*;
use crate::server::answer_cache::*;
use crate::gpt::gpt::*;
use crate::server::error::*;


struct AppState {
    counter: Mutex<u32>,
    client_factory: Arc<ClientsPool::<GptClient>>,
    answer_cache: StdMutex<AnswerCache>,
}

impl AppState {
    fn new(factory: Arc<dyn PollableClientFactory<GptClient> + Send + Sync>) -> Self {
        Self {
            counter: Mutex::new(0),
            client_factory: Arc::new(ClientsPool::<GptClient>::new(factory)),
            answer_cache: StdMutex::new(AnswerCache::new()),
        }
    }
}

type Shared = Arc<AppState>;

pub async fn run_server(
    factory: Arc<dyn PollableClientFactory<GptClient> + Send + Sync>,) -> anyhow::Result<()> {
    let state = Shared::new(AppState::new(factory));

    let app = Router::new()
        .route("/", get(index))
        .route("/gggame", get(gggame))
        .route("/ask", get(ask))
        .route("/answer/{token}", get(answer))
        .with_state(state)
        ;

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}

async fn gggame(ConnectInfo(_addr): ConnectInfo<SocketAddr>) -> String {
    format!("this is a gggame")
}

async fn answer(
    State(state): State<Shared>,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
    Path(token): Path<String>,
) -> String {
    match state.answer_cache.lock() {
        Ok(cache) => {
            match cache.get(&token) {
                AnswerCacheEntry::Text(text) => json!({
                    "answer": text,
                    "status": "ok"
                }).to_string(),
                AnswerCacheEntry::None => Er::status("invalid_token"),
                AnswerCacheEntry::Pending => Er::status("pending"),
            }
        }
        Err(_poisoned) => return "error".to_owned(),
    }
}

async fn ask(
    State(state): State<Shared>,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
) -> String {
    let token = match state.answer_cache.lock() {
        Ok(mut cache) => cache.reserve_token(),
        Err(_poisoned) => Er::error("internal server error")
    };

    let state2 = state.clone();
    let token_clone = token.clone();

    tokio::spawn(async move {
                let client_wrap = state2.client_factory.pop_client();
        let client = client_wrap.client();

        let mut params = QuestionParams::default();
        params.set_instructions("Short minimalistic answer");

        let result = match client.ask("Name a random well known actor.", &params).await {
            Ok(answer) => answer.to_string().unwrap_or_default(),
            Err(_) => Er::error("internal server error")
        };

        let mut cache = state2.answer_cache.lock().unwrap();
        cache.insert(&token_clone, &result);
    });

    json!({"token": token.to_owned(), "status": "ok"}).to_string()
}

async fn index(State(_state): State<Shared>,
             ConnectInfo(_addr): ConnectInfo<SocketAddr>) -> String {
    AnswerCache::generate_token()
}
