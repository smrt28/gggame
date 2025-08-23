#![allow(unused_imports)]

#[macro_use]

mod server;
mod gpt;

use std::path::PathBuf;
use std::sync::Arc;
use anyhow::Result;

use crate::server::server::run_server;
use crate::server::server::Config;
use crate::server::client_pool::*;
use crate::gpt::GptClient;
use tracing_subscriber::EnvFilter;

#[macro_use]
mod macros;
mod utinls;
mod token;
mod game_manager;

struct GptClientFactory {
    config: ClientFactoryConfig,
}

impl GptClientFactory {
    fn new() -> GptClientFactory {
        let mut res = Self {
            config: ClientFactoryConfig::default()
        };
        res.config.max_clients = 5;
        res
    }
}


impl PollableClientFactory::<GptClient> for GptClientFactory {
    fn build_client(&self) -> GptClient {
        let mut cli = GptClient::new();
        cli.read_gpt_key_from_file(None).expect("Can't read gpt API key");
        cli
    }

    fn get_config(&self) -> &ClientFactoryConfig {
        &self.config
    }
}

fn www_root() -> PathBuf {
    #[cfg(debug_assertions)]
    {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("www")
    }
    #[cfg(not(debug_assertions))]
    {
        PathBuf::from(std::env::var("WWW_ROOT")
            .expect("WWW_ROOT env var must be set in release builds"))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into())
        )
        .init();

   let mut config = Config::default();
    config.port = 3000;
    config.www_root_path = Some(www_root());
    run_server(&config, Arc::new(GptClientFactory::new())).await?;
    Ok(())
}

//  cargo build --release --target x86_64-unknown-linux-musl