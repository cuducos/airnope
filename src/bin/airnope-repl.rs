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

#[tokio::main(flavor = "multi_thread")]
pub async fn main() -> Result<()> {
    pretty_env_logger::init(); // based on RUST_LOG environment variable
    let embeddings = Arc::new(Mutex::new(Embeddings::new().await?));
    println!("Type `exit` to quit.");
    loop {
        let input = capture_input()?;
        if input == "exit" {
            break;
        }
        if is_spam(&embeddings, input.as_str()).await? {
            println!("Spam");
        } else {
            println!("Not spam");
        }
    }
    Ok(())
}
