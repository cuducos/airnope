use airnope::{embeddings::Embeddings, is_spam};
use anyhow::Result;
use std::{
    io::{stdin, stdout, Write},
    sync::Arc,
};
use tokio::sync::Mutex;

fn capture_input() -> Result<String> {
    let mut input = "".to_string();
    print!("> ");
    let _ = stdout().flush();
    stdin().read_line(&mut input)?;
    if let Some('\n') = input.chars().next_back() {
        input.pop();
    }
    if let Some('\r') = input.chars().next_back() {
        input.pop();
    }
    Ok(input)
}

pub async fn run() -> Result<()> {
    let embeddings = Arc::new(Mutex::new(Embeddings::new().await?));
    println!("Type `exit` to quit.");
    loop {
        let input = capture_input()?;
        if input == "exit" {
            break;
        }
        let result = is_spam(&embeddings, input.as_str()).await?;
        if result.is_spam {
            println!("Spam (score = {:.3})", result.score.unwrap_or(0.0));
        } else {
            println!("Not spam");
        }
    }
    Ok(())
}
