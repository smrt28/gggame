
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
use std::path::PathBuf;
use std::sync::Arc;
use axum::response::{Html, IntoResponse};
use tokio::{net::TcpListener, sync::Mutex};
use std::sync::Mutex as StdMutex;
use std::time::Duration;
use axum::extract::Query;
use axum::handler::Handler;
use axum::http::StatusCode;
use clap::builder::Str;
use serde::Deserialize;
use serde_json::json;
use crate::{gpt, GptClientFactory};
use crate::server::client_pool::*;
use crate::server::answer_cache::*;
use crate::gpt::gpt::*;
use crate::server::error::*;
use tokio::time::timeout;
use tower_http::services::ServeDir;
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, DefaultOnFailure};
use tracing::Level;
use tracing_subscriber::fmt::layer;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower::{ServiceBuilder};

#[derive(Deserialize)]
struct WaitParam { wait: Option<u64> }

struct AppState {
    counter: Mutex<u32>,
    client_factory: Arc<ClientsPool::<GptClient>>,
    answer_cache: StdMutex<AnswerCache>,
    config: Config,
}

#[derive(Default, Clone)]
pub struct Config {
    pub www_root_path: Option<PathBuf>,
    pub port: u16,
}

impl AppState {
    fn new(factory: Arc<dyn PollableClientFactory<GptClient> + Send + Sync>, config: &Config) -> Self {
        Self {
            counter: Mutex::new(0),
            client_factory: Arc::new(ClientsPool::<GptClient>::new(factory)),
            answer_cache: StdMutex::new(AnswerCache::new()),
            config: config.clone()
        }
    }
}

type Shared = Arc<AppState>;


fn logging() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO))
        .on_failure(DefaultOnFailure::new().level(Level::ERROR))
}

pub async fn run_server(
    config: &Config,
    factory: Arc<dyn PollableClientFactory<GptClient> + Send + Sync>,) -> anyhow::Result<()> {
    let state = Shared::new(AppState::new(factory, config));
    tracing::info!("starting server on port {}", config.port);

    let mut app = Router::new()
        .route("/", get(index))
        .route("/ask", get(ask))
        .route("/answer/{token}", get(answer))
        .fallback(get(handler_404))
        ;

    if let Some(root) = &config.www_root_path {
        let static_svc = ServiceBuilder::new()
            .layer(logging())
            .service(
                ServeDir::new(root)
                    .append_index_html_on_directories(true)
                    .precompressed_br()
                    .precompressed_gzip(),
            );
        app = app.nest_service("/static", static_svc);
    }

    app = app.layer(logging());

    let app = app
        .fallback(handler_404)
        .with_state(state)
        .layer(logging());

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let listener = TcpListener::bind(addr).await?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not found")
}

async fn answer(
    State(state): State<Shared>,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
    Path(token): Path<String>,
    Query(query): Query<WaitParam>,
) -> String {
    let wait = query.wait.unwrap_or(0);
    println!("wait: {}", wait);

    let snap = {
        let cache = state.answer_cache.lock().unwrap_or_else(|e| e.into_inner());
        match cache.get(&token) {
            AnswerCacheEntry::Text(text) => {
                return json!({ "status": "ok", "answer": text }).to_string();
            }
            AnswerCacheEntry::None => {
                return ErStatus::InvalidToken.json();
            }
            AnswerCacheEntry::Pending => cache.snapshot(&token), // Option<Slot>
        }
    };

    let Some(slot) = snap else {
        return ErStatus::InvalidToken.json();
    };

    let wait_secs = 3;
    if wait_secs > 0 {
        let _ = timeout(Duration::from_secs(wait_secs), slot.notify.notified()).await;
    } else {
        return ErStatus::Pending.json();
    }

    let entry_after = {
        let cache = state.answer_cache.lock().unwrap_or_else(|e| e.into_inner());
        cache.get(&token)
    };

    match entry_after {
        AnswerCacheEntry::Text(text) => json!({ "status": "ok", "answer": text }).to_string(),
        AnswerCacheEntry::None       => ErStatus::InvalidToken.json(),
        AnswerCacheEntry::Pending    => ErStatus::Pending.json(),
    }
}

async fn ask(
    State(state): State<Shared>,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
) -> String {
    let token = match state.answer_cache.lock() {
        Ok(mut cache) => cache.reserve_token(),
        Err(_poisoned) => ErStatus::error("internal server error").json()
    };

    let state2 = state.clone();
    let token_clone = token.clone();
    let wrap = state2.client_factory.pop();
    if !wrap.has_client() {
        return ErStatus::Overloaded.json();
    }

    tokio::spawn(async move {
        tracing::info!("token registered {}", token_clone);
        let client = wrap.client();

        let mut params = QuestionParams::default();
        params.set_instructions("Short minimalistic answer");

        let result = match client.ask("Name a random well known actor.", &params).await {
            Ok(answer) => answer.to_string().unwrap_or_default(),
            Err(_) => ErStatus::error("internal server error").json()
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
