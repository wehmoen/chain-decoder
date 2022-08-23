use std::error::Error;

use mongodb::{Client, Collection, Cursor, Database};
use mongodb::bson::{DateTime, doc};
use mongodb::options::UpdateOptions;
use mongodb::results::{InsertManyResult, UpdateResult};
use serde::{Deserialize, Serialize};
use crate::RRDecodedTransaction;

#[derive(Serialize, Deserialize)]
pub struct MetaData {
    pub key: String,
    pub value: String,
}

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
    metadata: Collection<MetaData>,
    transactions: Collection<Transaction>,
    decoded_transactions: Collection<RRDecodedTransaction>,
}

impl Adapter {
    pub async fn new(db_uri: &str, db_name: Option<&str>) -> Adapter {
        let client = Client::with_uri_str(db_uri).await.unwrap();
        let database = client.database(&db_name.unwrap_or("ronin".into()));
        let metadata = database.collection::<MetaData>("metadata");
        let transactions = database.collection::<Transaction>("transactions");
        let decoded_transactions = database.collection::<RRDecodedTransaction>("decoded_transactions");
        Adapter {
            client,
            database,
            metadata,
            transactions,
            decoded_transactions
        }
    }

    pub async fn metadata(&self, key: &str) -> mongodb::error::Result<Option<MetaData>> {
        self.metadata.find_one(doc! {
            "key": key
        }, None).await
    }

    pub async fn set_metadata(&self, key: String, value: String) -> mongodb::error::Result<UpdateResult> {
        let options = UpdateOptions::builder().upsert(Some(true)).build();
        self.metadata.update_one(doc! {
            "key": key
        },
        doc! {
            "value": value
        },
            options
        ).await
    }

    pub async fn transactions(&self, _last_block: i64) -> mongodb::error::Result<Cursor<Transaction>> {
        self.transactions.find(None, None).await
    }

    pub async fn insert_decoded(&self, decoded: &Vec<RRDecodedTransaction>) -> mongodb::error::Result<InsertManyResult> {
        self.decoded_transactions.insert_many(decoded, None).await
    }
}