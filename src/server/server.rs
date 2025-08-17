
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
use crate::{gpt, GptClientFactory};
use crate::server::client_pool::*;
use crate::server::answer_cache::*;

use crate::gpt::gpt::*;



struct AppState {
    counter: Mutex<u32>, // or AtomicU32 if it’s just a counter
    client_factory: Arc<ClientsPool::<GptClient>>,
    answer_cache: StdMutex<AnswerCache>,
}
impl AppState {
    fn new(factory: Arc<dyn PollableClientFactory<GptClient> + Send + Sync>) -> Self {
        Self {
            counter: Mutex::new(0),
            client_factory: Arc::new(ClientsPool::<GptClient>::new(factory)),
            answer_cache: Mutex::new(AnswerCache::new()),
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
    State(_state): State<Shared>,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
    Path(_token): Path<String>,
) -> String {
    "".to_string()
}

async fn ask(
    State(state): State<Shared>,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
) -> String {

    let client_wrap = state.client_factory.pop_client();
    let client = client_wrap.client();

    let cache = state.answer_cache.lock();
    if cache.is_err() {
        return "error".to_owned();
    }
    let token = cache.unwrap().reserve_token();

    let mut params = QuestionParams::default();
    params.set_instructions("Short minimalistic answer to the question. 1–2 words unless the correct name naturally requires more. No punctuation, no extra explanation.");

    let the_answer = match client.ask("Name a random well known actor.", &params).await {
        Ok(answer) => answer.to_string().unwrap_or_default(), // or just .to_string() if it returns String
        Err(_e) => "error".to_owned(),
    };

    format!("{}", the_answer)
}

async fn index(State(_state): State<Shared>,
             ConnectInfo(_addr): ConnectInfo<SocketAddr>) -> String
{
    AnswerCache::generate_token()
    //"index".to_string()
}
