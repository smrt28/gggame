#![allow(unused_imports)]

#[macro_use]

mod server;
mod gpt;

use std::sync::Arc;
use anyhow::{Result};

use crate::server::server::run_server;
use crate::server::server::Config;
use crate::server::client_pool::*;
use crate::gpt::gpt::GptClient;

#[macro_use]
mod macros;

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

#[tokio::main]
async fn main() -> Result<()> {
    let mut config = Config::default();
    config.port = 3000;
    run_server(&config, Arc::new(GptClientFactory::new())).await?;
    Ok(())
}
