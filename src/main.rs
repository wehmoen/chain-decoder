mod mongodb;
mod roninrest;

use futures::stream::{StreamExt, TryStreamExt};
use crate::mongodb::MetaData;
use crate::roninrest::RRDecodedTransaction;

#[tokio::main]
async fn main() {

    let db = mongodb::Adapter::new("mongodb://127.0.0.1:27017", Some("ronin")).await;
    let rr = roninrest::Adapter::new();

    let last_block = db.last_block().await;

    let mut txs = db.transactions(last_block).await.expect("Failed to create transaction cursor");

    let mut decoded :Vec<RRDecodedTransaction> = vec![];

    let mut total_decoded: i128 = 0;

    while let Some(tx) = txs.try_next().await.unwrap() {
        println!("Pending decoded TX: {}", decoded.len());
        decoded.push(RRDecodedTransaction {
            from: tx.from,
            to: tx.to,
            block_number: tx.block as u64,
            input: Some(rr.decode_method(&tx.hash).await),
            output: Some(rr.decode_receipt(&tx.hash).await),
            hash: tx.hash,
        });

        if decoded.len() >= 5000 {
            total_decoded = total_decoded + decoded.len() as i128;
            db.insert_decoded(&decoded).await.expect("Failed to insert into db!");
            decoded.clear();
            println!("{} complete!", total_decoded);
        }
    }

    if decoded.len() > 0 {
        total_decoded = total_decoded + decoded.len() as i128;
        db.insert_decoded(&decoded).await.expect("Failed to insert into db!");
        decoded.clear();
        println!("{} complete!", total_decoded);
    }

    println!("DONE!")

}
