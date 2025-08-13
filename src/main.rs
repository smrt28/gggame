#![allow(unused_imports)]

#[macro_use]

mod server;
mod gpt;


use anyhow::{Result};
use crate::server::run_server;
use crate::gpt::*;

#[macro_use]
mod macros;

#[tokio::main]
async fn main() -> Result<()> {
/*

    let mut params = QuestionParams::default();
    params.set_instructions("Short minimalistic answer to the question. 1–2 words unless the correct name naturally requires more. No punctuation, no extra explanation.");


    cli.read_gpt_key_from_file(None)?;
    //let answer = cli.ask("Where is Prague located?", &params).await?;
    let answer = cli.ask("In order to play the game 'guess the animal', Choose the animal by random you are going to be and tell me the animal.", &params).await?;
    let res = answer.to_string().unwrap_or(String::new());
    println!("{}", res);

    answer.dump();

 */
/*
    let mut cli = GptClient::new();
    cli.read_gpt_key_from_file(None)?;

    let mut params = gpt::QuestionParams::default();
    //params.set_temperature(1.5);
    params.set_instructions("Short minimalistic answer to the question. 1–2 words unless the correct name naturally requires more. No punctuation, no extra explanation.");
    let answer = cli.ask("Name a random well known actor.", &params).await?;
    let res = answer.to_string().unwrap_or(String::new());
    println!("{}", res);
*/
    run_server().await?;

    Ok(())
}