use axum::{extract::ConnectInfo, routing::get, Router, extract::State};

use std::net::SocketAddr;
use std::sync::Arc;
use axum::response::Html;
use tokio::{net::TcpListener, sync::Mutex};

#[derive(Default)]
struct AppState {
    counter: Mutex<u32>,

}

type Shared = Arc<AppState>;

pub async fn run_server() -> anyhow::Result<()> {
    let state = Shared::new(AppState::default());

    let app = Router::new()
        .route("/", get(index))
        .route("/gggame", get(gggame))
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
    #[allow(unused_assignments)]
    let mut counter_value = 0;

    {
        let mut counter = state.counter.lock().await;
        *counter +=1;
        counter_value = *counter;
    }

    let html = format!(
        "<h1>Your ip address is: \"{addr} counter={counter_value}\"</h1>\n"
    );

    // create `Html` type like this
    Html(html)
}