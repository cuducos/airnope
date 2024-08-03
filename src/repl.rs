use anyhow::Result;
use std::io::{stdin, stdout, Write};

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
    let pipeline = crate::Pipeline::new().await?;
    println!("Type `exit` to quit.");
    loop {
        let input = capture_input()?;
        if input == "exit" {
            break;
        }
        if pipeline.is_spam(input).await? {
            println!("Spam");
        } else {
            println!("Not spam");
        }
    }
    Ok(())
}
