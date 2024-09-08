use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
    str::FromStr,
};

use non_empty_string::NonEmptyString;
use url::Url;

#[derive(clap::Parser)]
pub struct CliArgs {
    #[arg(long, default_value_t = GitHostChoice::GitHub)]
    pub githost: GitHostChoice,

    #[arg(long)]
    pub app_id: usize,

    #[arg(long, default_value_t = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))]
    pub webhook_addr: IpAddr,

    #[arg(long, default_value_t = 8099)]
    pub webhook_server_port: u16,

    #[arg(long)]
    pub pem_rsa_key_path: PathBuf,

    #[arg(long, default_value_t = LlmChoice::OpenAi)]
    pub llm: LlmChoice,

    #[arg(long, default_value_t = Url::from_str("https://api.openai.com/v1").unwrap())]
    pub llm_api_base_url: Url,

    #[arg(long, default_value = "gpt-4o-mini")]
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

impl ToString for LlmChoice {
    fn to_string(&self) -> String {
        match self {
            LlmChoice::OpenAi => "open-ai".into(),
        }
    }
}

#[derive(Clone, clap::ValueEnum)]
pub enum GitHostChoice {
    GitHub,
    // :)
}

impl ToString for GitHostChoice {
    fn to_string(&self) -> String {
        match self {
            GitHostChoice::GitHub => "git-hub".into(),
        }
    }
}
