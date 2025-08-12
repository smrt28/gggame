
#[macro_use]
mod string_enum;
mod gpt;
mod server;

use anyhow::{Context, Result};

use crate::server::run_server;





#[tokio::main]
async fn main() -> Result<()> {
/*
    let mut cli = GptClient::new();
    let mut params = QuestionParams::default();
    params.set_instructions("Short minimalistic answer to the question. 1–2 words unless the correct name naturally requires more. No punctuation, no extra explanation.");


    cli.read_gpt_key_from_file(None)?;
    //let answer = cli.ask("Where is Prague located?", &params).await?;
    let answer = cli.ask("In order to play the game 'guess the animal', Choose the animal by random you are going to be and tell me the animal.", &params).await?;
    let res = answer.to_string().unwrap_or(String::new());
    println!("{}", res);

    answer.dump();

 */
    run_server().await?;

    Ok(())
}