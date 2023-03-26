use std::future::Future;

use clap::Parser;
use serde_json::json;
use tokio;
use types::Message;
mod types;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short = 'm', long = "message")]
    message: String,
    #[arg(short = 'i', long = "id")]
    chat_id: Option<String>,
}

async fn apply_async<T, U, F, Fut>(x: Option<T>, g: F) -> Option<U>
where
    F: Fn(T) -> Fut,
    Fut: Future<Output = U>,
    U: Sized,
{
    return Some(x.map(|y| g(y))?.await);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dote = dotenv::dotenv().ok();
    let args = Cli::parse();
    // Only have save_path if we have an id; otherwise it's none (so we can have empty context)
    let save_path = args
        .chat_id
        .as_ref()
        .map(|id| std::env::var("SAVE_PATH").unwrap() + &id + ".json");
    let token = std::env::var("CGPT_TOKEN").unwrap();
    println!("{:?}", &args);

    let msg = types::Message {
        role: types::Role::User,
        content: args.message,
    };

    // Load historic_conversion or set it to empty
    // empty if no save_path OR no save_file
    let mut previous_chat = match &save_path {
        Some(p) => tokio::fs::read(&p).await.map_or(vec![], |y| {
            serde_json::from_slice::<Vec<Message>>(&y).unwrap()
        }),
        _ => vec![],
    };
    previous_chat.push(msg);

    let answer = chat_gpt(&previous_chat, &token).await.unwrap();
    previous_chat.push(TryInto::<Message>::try_into(answer.clone()).unwrap());
    println!("ChatGPT: \n{}", &previous_chat.last().unwrap().content);

    match &save_path {
        Some(p) => save_json(&previous_chat, &p).await.unwrap(),
        _ => (),
    };
    return Ok(());
}

async fn save_json<T, P>(content: &T, path: P) -> Result<(), std::io::Error>
where
    T: serde::Serialize, // Clone
    P: AsRef<std::path::Path>,
{
    return tokio::fs::write(path, serde_json::to_string(&content).unwrap()).await;
}

async fn chat_gpt<T>(
    messages: &Vec<types::Message>,
    token: &T,
) -> Result<types::MessageResponse, Box<dyn std::error::Error>>
where
    T: std::fmt::Display,
{
    const gpt_url: &str = "https://api.openai.com/v1/chat/completions";

    let client = reqwest::Client::new();
    let json_content = json!({
        "model": "gpt-3.5-turbo",
        "messages": messages,
        "temperature": 0.5
    });

    let request = client
        .post(gpt_url)
        .bearer_auth(token)
        .json(&json_content);

    let response = request.send().await.unwrap();
    return Ok(response.json().await.unwrap());
}
