use std::ops::Deref;

use derive_more::derive::{AsRef, Deref, From, FromStr, Into};
use non_empty_string::NonEmptyString;

pub enum ChatMessage {
    UserMessage(UserMessage),
    AiMessage(AiMessage),
}

impl AsRef<NonEmptyString> for ChatMessage {
    fn as_ref(&self) -> &NonEmptyString {
        match self {
            ChatMessage::UserMessage(user_message) => user_message.as_ref(),
            ChatMessage::AiMessage(user_message) => user_message.as_ref(),
        }
    }
}

impl Deref for ChatMessage {
    type Target = NonEmptyString;

    fn deref(&self) -> &Self::Target {
        match self {
            ChatMessage::UserMessage(user_message) => user_message.deref(),
            ChatMessage::AiMessage(user_message) => user_message.deref(),
        }
    }
}

#[derive(FromStr, AsRef, Deref, Into, From)]
pub struct UserMessage(NonEmptyString);

impl From<UserMessage> for ChatMessage {
    fn from(val: UserMessage) -> Self {
        ChatMessage::UserMessage(val)
    }
}

#[derive(FromStr, AsRef, Deref, Into, From)]
pub struct AiMessage(NonEmptyString);

impl From<AiMessage> for ChatMessage {
    fn from(val: AiMessage) -> Self {
        ChatMessage::AiMessage(val)
    }
}
