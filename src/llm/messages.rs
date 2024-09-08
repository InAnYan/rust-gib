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

impl Into<ChatMessage> for UserMessage {
    fn into(self) -> ChatMessage {
        ChatMessage::UserMessage(self)
    }
}

#[nutype(validate(not_empty), derive(FromStr, AsRef, Deref))]
pub struct AiMessage(String);

impl Into<ChatMessage> for AiMessage {
    fn into(self) -> ChatMessage {
        ChatMessage::AiMessage(self)
    }
}
