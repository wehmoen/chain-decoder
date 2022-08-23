use std::time::Duration;

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use serde::{Deserialize, Serialize};

const DEFAULT_USER_AGENT: &str = "ronin/wally0.1.0 See: https://github.com/wehmoen/ronin-wally";

pub type RRTransactionHash = String;

#[derive(Serialize, Deserialize)]
pub struct RRTransactionDict {
    pub transactions: Vec<RRTransactionHash>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RRTransaction {
    pub from: String,
    pub to: String,
    pub hash: String,
    pub block_number: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RRDecodedTransaction {
    pub from: String,
    pub to: String,
    pub hash: RRTransactionHash,
    pub block_number: u64,
    pub input: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
}

fn normalize_address(input: &str) -> String {
    input.replace("ronin:", "0x")
}

pub struct Adapter {
    pub host: String,
    client: ClientWithMiddleware,
}

impl Adapter {
    pub fn new() -> Adapter {
        Adapter {
            host: "https://ronin.rest".into(),
            client: ClientBuilder::new(reqwest::Client::new()).with(
                RetryTransientMiddleware::new_with_policy(
                    ExponentialBackoff {
                        max_n_retries: 25,
                        min_retry_interval: Duration::from_secs(1),
                        max_retry_interval: Duration::from_secs(15),
                        backoff_exponent: 2,
                    }
                )
            ).build(),
        }
    }

    pub async fn decode_method(&self, hash: &RRTransactionHash) -> serde_json::Value {
        let data: serde_json::Value = serde_json::from_str(
            &self.client.get(format!("{}/ronin/decodeTransaction/{}", self.host, hash)).header("user-agent", DEFAULT_USER_AGENT).send().await.unwrap().text().await.unwrap()
        ).unwrap();

        data
    }

    pub async fn decode_receipt(&self, hash: &RRTransactionHash) -> serde_json::Value {
        let data: serde_json::Value = serde_json::from_str(
            &self.client.get(format!("{}/ronin/decodeTransactionReceipt/{}", self.host, hash)).header("user-agent", DEFAULT_USER_AGENT).send().await.unwrap().text().await.unwrap()
        ).unwrap();

        data
    }
}

pub struct ArgParser {}

impl ArgParser {
    fn parse() -> Vec<String> {
        std::env::args().collect()
    }

    pub fn split(param: &String) -> Option<String> {
        let args: Vec<String> = ArgParser::parse();

        for arg in args {
            if arg.starts_with(param) {
                let kv: Vec<&str> = arg.split('=').collect();
                if kv.len() == 2 {
                    return Some(kv[1].to_string());
                }
            }
        }

        None
    }
}
