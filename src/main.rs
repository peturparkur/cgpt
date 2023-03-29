use std::{future::Future, path::{Path, PathBuf}, fmt::{Debug, Display}};

use clap::{Parser, Subcommand};
use serde::{Serialize, Deserialize};
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

#[derive(Debug, Parser)]
#[command(about = "CLI app for easy calling of ChatGPT")]
struct CliArgs {
    #[command(subcommand)]
    command: Commands
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true, about = "opens a previous conversation")]
    Checkout {
        #[arg(short = 'i', long = "id")]
        id: String,
        #[arg(short = 'p', long = "prompt")]
        prompt: Option<String>
    },
    #[command(arg_required_else_help = true, about = "continue conversation")]
    Message {
        #[arg(short = 'm', long = "message")]
        message: String, 
        #[arg(short = 's', long = "nosave")]
        no_save: Option<bool>
    },
    #[command(arg_required_else_help = true, about = "send message without history")]
    Ask {
        #[arg(short = 'm', long = "message")]
        message: String,
        #[arg(short = 'p', long = "prompt")]
        prompt: Option<String>
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Configuration {
    save_path: PathBuf,
    current_chat: String,
}
impl Default for Configuration {
    fn default() -> Self {
        let binding = std::env::var("HOME").unwrap();
        let p = Path::new(&binding);
        let fp = p.join(PathBuf::from(".config/cgpt/conversations"));
        Self {
            save_path: fp,
            current_chat: "chat".to_string()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // let dote = dotenv::dotenv().ok(); // Only for debugging
    let binding = std::env::var("HOME").unwrap();
    let base_path = Path::new(&binding);
    let config_path = base_path.join(PathBuf::from(".config/cgpt/.cf"));
    let cfg = match confy::load_path::<Configuration>(&config_path) {
        Ok(x) => Ok(x),
        Err(confy::ConfyError::ReadConfigurationFileError(x)) => Ok(Configuration::default()),
        Err(x) => Err(x)
    }
    .unwrap();
    println!("configuration: {:?}", &cfg);
    let token = std::env::var("CGPT_TOKEN").unwrap();



    let clap_args = CliArgs::parse();
    let y = match clap_args.command {
        Commands::Checkout { id, prompt } => {
            confy::store_path(&config_path, Configuration{save_path: cfg.save_path, current_chat: id}).unwrap();
            ()
        },
        Commands::Message { message, no_save } => {
            let msg = Message{role: types::Role::User, content: message};
            let context: Vec<Message> = vec![];
            ()
        },
        Commands::Ask { message, prompt } => {
            let msg = Message{role: types::Role::User, content: message};
            let background = prompt
                .map(|x| Message{role: types::Role::System, content: x})
                .map_or(vec![msg.clone()], |x| vec![x, msg.clone()]);
            let answer = chat_gpt(&background, &token).await.unwrap();
            println!("ChatGPT: \n{}", &answer.choices.first().unwrap().message.content);
            ()
        }
    };
    return Ok(())
}

async fn save_json<T, P>(content: &T, path: &P) -> Result<(), std::io::Error>
where
    T: serde::Serialize, // Clone
    P: AsRef<std::path::Path> + Debug,
{
    println!("Attempt saving to: {:?}", &path);
    let _tmp = tokio::fs::create_dir(path).await?;
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
