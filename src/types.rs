use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Usage {
    pub completion_tokens: u32,
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct MessageChoice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct MessageResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub choices: Vec<MessageChoice>,
    pub usage: Usage,
}

impl Into<Vec<Message>> for MessageResponse {
    fn into(self) -> Vec<Message> {
        return self
            .choices
            .into_iter()
            .map(|x| x.message)
            .collect::<Vec<Message>>();
    }
}
impl TryInto<Message> for MessageResponse {
    type Error = String;
    fn try_into(self) -> Result<Message, Self::Error> {
        return Into::<Vec<Message>>::into(self)
            .into_iter()
            .next()
            .map_or(Err("No Element".into()), |x| Ok(x));
    }
}
