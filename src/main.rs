#![allow(unused_imports)]

#[macro_use]

mod server;
mod gpt;

use std::sync::Arc;
use anyhow::{Result};

use crate::server::server::run_server;
use crate::server::client_pool::*;
use crate::gpt::gpt::GptClient;

#[macro_use]
mod macros;


struct GptClientFactory {

}

impl PollableClientFactory::<GptClient> for GptClientFactory {
    fn build_client(&self) -> GptClient {
        let mut cli = GptClient::new();
        cli.read_gpt_key_from_file(None).unwrap();
        cli
    }
}


#[tokio::main]
async fn main() -> Result<()> {
    run_server(Arc::new(GptClientFactory {})).await?;
    Ok(())
}