use std::fmt::Debug;
use std::{marker::PhantomData, path::PathBuf};

use log::debug;
use non_empty_string::NonEmptyString;
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};
use tracing::instrument;

use super::{
    llm_trait::{CompletionParameters, Llm},
    messages::UserMessage,
};

#[derive(Debug, thiserror::Error)]
pub enum LlmAgentError<LE> {
    #[error("error from template engine")]
    TemplateEngineError(#[from] tera::Error),

    #[error("rendered template is empty")]
    TemplateRenderIsEmptyError,

    #[error("error from LLM")]
    LlmError(#[source] LE), // We cannot use `#[from]` here because of https://github.com/dtolnay/thiserror/issues/323
}

pub type Result<T, LE> = std::result::Result<T, LlmAgentError<LE>>;

#[derive(Serialize, Deserialize)]
pub struct LlmAgentConfig {
    system_message_template_path: PathBuf,
    user_message_template_path: PathBuf,
    completion_params: CompletionParameters,
}

pub struct LlmAgent<L, C> {
    llm: L,
    template_engine: Tera,
    completion_params: CompletionParameters,
    _context: PhantomData<C>,
}

pub const CONTEXT_VAR_NAME: &str = "context";

const SYSTEM_MESSAGE_TEMPLATE_NAME: &str = "system_message";
const USER_MESSAGE_TEMPLATE_NAME: &str = "user_message";

impl<L: Llm, C: Serialize + Debug> LlmAgent<L, C> {
    pub fn build_from_config(llm: L, config: LlmAgentConfig) -> Result<Self, L::Error> {
        let mut template_engine = Tera::default();

        template_engine.add_template_files(vec![
            (
                config.system_message_template_path,
                Some(SYSTEM_MESSAGE_TEMPLATE_NAME),
            ),
            (
                config.user_message_template_path,
                Some(USER_MESSAGE_TEMPLATE_NAME),
            ),
        ])?;

        Ok(Self {
            llm,
            template_engine,
            completion_params: config.completion_params,
            _context: PhantomData,
        })
    }

    pub fn build_raw(
        llm: L,
        system_message_template: NonEmptyString,
        user_message_template: NonEmptyString,
        completion_params: CompletionParameters,
    ) -> Result<Self, L::Error> {
        let mut template_engine = Tera::default();

        template_engine.add_raw_template(
            SYSTEM_MESSAGE_TEMPLATE_NAME,
            system_message_template.as_ref(),
        )?;

        template_engine
            .add_raw_template(USER_MESSAGE_TEMPLATE_NAME, user_message_template.as_ref())?;

        Ok(Self {
            llm,
            template_engine,
            completion_params,
            _context: PhantomData,
        })
    }

    #[instrument(skip(self))]
    pub async fn process(&self, context: &C) -> Result<NonEmptyString, L::Error> {
        let mut tera_context = Context::new();
        tera_context.insert(CONTEXT_VAR_NAME, &context);

        let system_message: NonEmptyString = self
            .template_engine
            .render(SYSTEM_MESSAGE_TEMPLATE_NAME, &tera_context)?
            .try_into()
            .map_err(|_| LlmAgentError::TemplateRenderIsEmptyError)?;

        debug!("Rendered system message:\n{}", system_message);

        let user_message: NonEmptyString = self
            .template_engine
            .render(USER_MESSAGE_TEMPLATE_NAME, &tera_context)?
            .try_into()
            .map_err(|_| LlmAgentError::TemplateRenderIsEmptyError)?;

        debug!("Rendered user message:\n{}", user_message);

        let ai_message: NonEmptyString = self
            .llm
            .complete(
                &system_message,
                vec![UserMessage::from(user_message).into()],
                &self.completion_params,
            )
            .await
            .map_err(LlmAgentError::LlmError)?
            .into();

        debug!("Resulting AI message:\n{}", ai_message);

        Ok(ai_message)
    }
}
