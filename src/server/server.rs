
#![allow(dead_code)]

use axum::{extract::ConnectInfo, routing::get, Router, extract::State};
use anyhow::{Context, Error, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use axum::response::Html;
use tokio::{net::TcpListener, sync::Mutex};
use std::sync::Mutex as StdMutex;

use crate::server::client_pool::*;
use crate::gpt::*;
use crate::GptClientFactory;

struct AppState {
    counter: Mutex<u32>, // or AtomicU32 if it’s just a counter
    client_factory: Arc<dyn PollableClientFactory<GptClient> + Send + Sync>,
}
impl AppState {
    fn new(factory: Arc<dyn PollableClientFactory<GptClient> + Send + Sync>) -> Self {
        Self { counter: Mutex::new(0), client_factory: factory }
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
        .with_state(state)
        ;

    //app.route("/gggame", get(gggame));

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

async fn index(
    State(state): State<Shared>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>) -> Html<String> {


    let client = state.client_factory.build_client();

    let counter_value = {
        let mut counter = state.counter.lock().await;
        *counter +=1;
        *counter
    };


    let html = format!(
        "<h1>Your ip address is: \"{addr} counter={}\"</h1>\n",
        counter_value
    );

    // create `Html` type like this
    Html(html)
}

async fn ask(State(_state): State<Shared>,
             ConnectInfo(_addr): ConnectInfo<SocketAddr>) -> String
{
    let r: anyhow::Result<String> = async {
        let mut cli = GptClient::new();
        cli.read_gpt_key_from_file(None)?; // OK: block returns Result

        let mut params = QuestionParams::default();
        params.set_instructions(
            "Short minimalistic answer to the question. 1–2 words unless the correct name naturally requires more. No punctuation, no extra explanation."
        );

        let answer = cli.ask("Name a random well known actor.", &params).await?; // OK
        Ok(answer.to_string().unwrap_or_default()) // <-- return Result; no semicolon
    }.await;

    if let Ok(r) = r {
        return r;
    }

    "error".to_string()
}
