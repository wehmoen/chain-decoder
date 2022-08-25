use std::error::Error;

use mongodb::{Client, Collection, Cursor, Database};
use mongodb::bson::{DateTime, doc};
use mongodb::options::{FindOneOptions, FindOptions, UpdateOptions};
use mongodb::results::{InsertManyResult, UpdateResult};
use serde::{Deserialize, Serialize};

use crate::RRDecodedTransaction;

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub hash: String,
    pub block: u32,
    pub created_at: Option<DateTime>,
}

pub struct Adapter {
    client: Client,
    database: Database,
    transactions: Collection<Transaction>,
    decoded_transactions: Collection<RRDecodedTransaction>,
}

impl Adapter {
    pub async fn new(db_uri: &str, db_name: Option<&str>) -> Adapter {
        let client = Client::with_uri_str(db_uri).await.unwrap();
        let database = client.database(&db_name.unwrap_or("ronin".into()));
        let transactions = database.collection::<Transaction>("transactions");
        let decoded_transactions = database.collection::<RRDecodedTransaction>("decoded_transactions");
        Adapter {
            client,
            database,
            transactions,
            decoded_transactions,
        }
    }

    pub async fn last_block(&self) -> u64 {
        let options = FindOneOptions::builder().sort(doc! {
            "blockNumber": -1
        }).build();
        match self.decoded_transactions.find_one(None, options).await.unwrap() {
            None => 0,
            Some(tx) => tx.block_number
        }
    }

    pub async fn transactions(&self, last_block: u64) -> mongodb::error::Result<Cursor<Transaction>> {
        let options = FindOptions::builder()
            .no_cursor_timeout(Some(true))
            .batch_size(Some(100u32))
            .build();
        self.transactions.find(doc! {
            "block": {
                "&gt": last_block as i64
            }
        }, options).await
    }

    pub async fn insert_decoded(&self, decoded: &Vec<RRDecodedTransaction>) -> mongodb::error::Result<InsertManyResult> {
        self.decoded_transactions.insert_many(decoded, None).await
    }
}