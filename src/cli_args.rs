use std::net::IpAddr;

use non_empty_string::NonEmptyString;
use url::Url;

#[derive(clap::Parser)]
pub struct CliArgs {
    #[arg(long, group = "Git host")]
    pub githost: GitHostChoice,

    #[arg(long, group = "Git host")]
    pub app_id: usize,

    #[arg(long, group = "Git host")]
    pub webhook_addr: IpAddr,

    #[arg(long, group = "Git host")]
    pub webhook_server_port: u16,

    #[arg(long, group = "LLM")]
    pub llm: LlmChoice,

    #[arg(long, group = "LLM")]
    pub llm_api_base_url: Url,

    #[arg(long, group = "LLM")]
    pub llm_model: NonEmptyString,

    #[arg(long, action)]
    pub allow_list: bool,

    #[arg(long)]
    pub features: Vec<NonEmptyString>,
}

#[derive(Clone, clap::ValueEnum)]
pub enum LlmChoice {
    OpenAi,
    // :)
}

#[derive(Clone, clap::ValueEnum)]
pub enum GitHostChoice {
    GitHub,
    // :)
}
