use clap::Parser;

use std::str::FromStr;
use std::string::ToString;

#[derive(Parser)]
#[clap(version = "0.2.0", author = "trey.dempsey@gmail.com")]
pub struct CliOptions {
    #[clap(
        short,
        long,
        default_value = "localhost:9092",
        about = "Kafka bootstrap servers connect string"
    )]
    pub bootstrap_servers: String,
    #[clap(short, long, about = "Kafka topic to monitor")]
    pub topic: String,
    #[clap(short, long, default_value = "kafka-toy", about = "Consumer group id")]
    pub group_id: String,
    #[clap(
        short,
        long,
        default_value = "latest",
        about = "Topic offset (auto.offset.reset) for new consumer group ids [earliest|latest]"
    )]
    pub offset: Offset,
    #[clap(about = "JQ style JSON filter to apply to each message")]
    pub filter: String,
}

pub enum Offset {
    Earliest,
    Latest,
}

impl FromStr for Offset {
    type Err = anyhow::Error;

    fn from_str(arg: &str) -> anyhow::Result<Self> {
        match arg.to_lowercase().as_str() {
            "earliest" => Ok(Self::Earliest),
            "latest" => Ok(Self::Latest),
            _ => Err(anyhow::anyhow!(
                "Invalid offset: {}. Must be one of earliest or latest",
                arg
            )),
        }
    }
}

impl ToString for Offset {
    fn to_string(&self) -> String {
        match self {
            Self::Earliest => "earliest".to_owned(),
            Self::Latest => "latest".to_owned(),
        }
    }
}
