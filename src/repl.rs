use anyhow::Result;
use std::{
    io::{stdin, stdout, Write},
    sync::Arc,
};
use tokio::sync::Mutex;

use crate::embeddings::Embeddings;

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

pub async fn run(embeddings: &Arc<Mutex<Embeddings>>) -> Result<()> {
    println!("Type `exit` to quit.");
    loop {
        let input = capture_input()?;
        if input == "exit" {
            break;
        }
        if crate::is_spam(embeddings, input.as_str()).await? {
            println!("Spam");
        } else {
            println!("Not spam");
        }
    }
    Ok(())
}
