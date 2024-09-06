use std::ops::Deref;

use nutype::nutype;

pub enum ChatMessage {
    UserMessage(UserMessage),
    AiMessage(AiMessage),
}

impl AsRef<str> for ChatMessage {
    fn as_ref(&self) -> &str {
        match self {
            ChatMessage::UserMessage(user_message) => user_message.as_ref(),
            ChatMessage::AiMessage(user_message) => user_message.as_ref(),
        }
    }
}

impl Deref for ChatMessage {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            ChatMessage::UserMessage(user_message) => user_message.deref(),
            ChatMessage::AiMessage(user_message) => user_message.deref(),
        }
    }
}

#[nutype(validate(not_empty), derive(FromStr, AsRef, Deref))]
pub struct UserMessage(String);

#[nutype(validate(not_empty), derive(FromStr, AsRef, Deref))]
pub struct AiMessage(String);
