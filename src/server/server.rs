
#![allow(dead_code)]


use axum::{extract::ConnectInfo, routing::get, Router, extract::State};
use anyhow::{Context, Error, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use axum::response::Html;
use tokio::{net::TcpListener, sync::Mutex};
use std::sync::Mutex as StdMutex;

use crate::gpt::*;

#[derive(Default)]
struct AppState {
    counter: Mutex<u32>,
}


trait PollableClientFactory<Client> : Send + Sync {
    fn build_client(&self) -> Client;
}

struct ClientsPool<Client> {
    clients: StdMutex<Vec<Box<Client>>>,
    factory: Box<dyn PollableClientFactory<Client> + Send + Sync>,
}

struct ClientGuard<Client> {
    client: Option<Box<Client>>,
    pool: Arc<ClientsPool<Client>>,
}

impl<Client> Drop for ClientGuard<Client>
{
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            self.pool.return_client(client);
        }
    }
}

impl<Client> ClientsPool<Client> {
    fn new(factory: Box<dyn PollableClientFactory<Client>>) -> Self {
        Self {
            clients: StdMutex::new(Vec::new()),
            factory: factory
        }
    }

    fn pop_client(self: &Arc<Self>) -> ClientGuard<Client> {
        let mut clients = self.clients.lock().unwrap();
        if clients.len() == 0 {
            return ClientGuard {
                client: Some(Box::new(self.factory.build_client())),
                pool: Arc::clone(self)
            };
        }

        let client = clients.pop().unwrap();
        ClientGuard { client: Some(client), pool: Arc::clone(self) }
    }

    fn return_client(&self, client: Box<Client>) {
        if let Ok(mut clients) = self.clients.lock() {
            clients.push(client);
        }
    }
}




impl AppState {
    /*
    async fn get_gpt_client(&mut self) -> Result<GptClient> {
        {
            let mut clients = self.gpt_clients.lock().await;
            if (*clients).len() > 0 {
                return (*clients).pop().ok_or(Error::msg("no gpt client available"));
            }
        }

        let mut client = GptClient::new();
        client.read_gpt_key_from_file(None)?;
        Ok(client)
    }
    */

}

type Shared = Arc<AppState>;

pub async fn run_server() -> anyhow::Result<()> {
    let state = Shared::new(AppState::default());

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

async fn ask(State(state): State<Shared>,
             ConnectInfo(addr): ConnectInfo<SocketAddr>) -> String
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
