use std::future::Future;
use std::io::Cursor;
use std::option::Option;
use serde::Deserialize;
use bytes::Bytes;
use atoi::atoi;
use reqwest::Response;
use base64::{decode_config, URL_SAFE};


#[derive(Deserialize, Debug)]
struct Bundle {
    format: i32,
    id: String,
    last_tx: String,
    owner: String,
    target: String,
    quantity: String,
    data: String,
    data_size: String,
    data_root: String,
    reward: String,
    signature: String
}

#[derive(Deserialize, Debug, Clone)]
struct Chunk {
    tx_path: String,
    packing: String,
    data_path: String,
    chunk: String
}

#[derive(Deserialize, Debug)]
struct Offset {
    size: String,
    offset: String,
}

struct Peers {

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let id = "F8Bcp5-dfOhRnOZm7dev58uRKmiXkn9my4d6WnkyPDU";
    let offset = reqwest::get(format!("https://arweave.net/tx/{}/offset", id))
        .await
        .unwrap()
        .json::<Offset>()
        .await
        .unwrap();

    let mut offset_size = atoi::<i64>(offset.offset.as_bytes()).unwrap();
    let mut tx_size = atoi::<i64>(offset.size.as_bytes()).unwrap();

    let chunk_size = 256 * 1024;
    let mut known_peers: Vec<String> = Vec::new();
    known_peers.push(String::from("https://arweave.net"));

    let mut b: Vec<u8> = Vec::new();
    while tx_size > 0 {
        println!("offset size {} tx left {} chunk size {}", offset_size, tx_size, chunk_size);
        let resp: Response = reqwest::get(format!("https://arweave.net/chunk/{}", offset_size)).await?;
        let json_chunk: Chunk = match resp.json::<Chunk>().await {
            Ok(x) => x,
            Err(_e) => {
                panic!("{}", format!("could not find chunk {}", offset_size))
            },
        };

        let x = match decode_config(json_chunk.chunk, URL_SAFE) {
            Ok(res) => b.extend(res),
            Err(e) => {
                panic!("{}", format!("Not able to decode chunk {}: {}", offset_size, e))
            },
        };

        offset_size -= chunk_size;
        tx_size -= chunk_size
    }

    // Assume chunk is in text format
    // const chunk_size = Bytes::from(chunk);
    Ok(())
}

// format
// id
// last_tx
// owner
// tags
// target
// quantity
// data