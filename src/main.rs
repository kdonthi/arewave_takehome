use std::ffi::CString;
use std::future::Future;
use std::io::Cursor;
use std::option::Option;
use serde::Deserialize;
use bytes::Bytes;
use atoi::atoi;
use reqwest::Response;
use base64::{decode_config, URL_SAFE};


#[derive(Deserialize, Debug)]
struct Tag {
    name: String,
    value: String
}

#[derive(Deserialize, Debug)]
struct Transaction {
    format: i32,
    id: String,
    last_tx: String,
    owner: String,
    tags: Vec<Tag>,
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

#[derive(Deserialize, Debug)]
struct AvroField {
    name: String,
    #[serde(alias = "type")]
    typ: String
}

#[derive(Deserialize, Debug)]
struct AvroItems {
    #[serde(alias = "type")]
    typ: String,
    name: String,
    fields: Vec<AvroField>
}

#[derive(Deserialize, Debug)]
struct AvroArray {
    #[serde(alias = "type")]
    typ: String,
    items: AvroItems
}

#[derive(Deserialize, Debug)]
struct DataItem {
    signature_type: String,
    signature: String,
    owner: String,
    target: String,
    anchor: String,
    tag_count: i32,
    tag_bytes: i32,
    tags: AvroArray,
    data: Vec<u8>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let id = "F8Bcp5-dfOhRnOZm7dev58uRKmiXkn9my4d6WnkyPDU";

    let tx = reqwest::get(format!("https://arweave.net/tx/{}", id))
        .await
        .unwrap()
        .json::<Transaction>()
        .await
        .unwrap();

    let mut seen_bundle_format = false;
    let mut seen_bundle_version = false;

    for tag in tx.tags {
        let name_bytes = decode_config(tag.name, URL_SAFE).unwrap();
        let name = String::from_utf8(name_bytes).unwrap();

        if name == "Bundle-Format" {
            seen_bundle_format = true;
        }
        if name == "Bundle-Version" {
            seen_bundle_version = true;
        }
    }

    if !seen_bundle_version || !seen_bundle_format {
        panic!("transaction id given is not a bundle");
    }

    let offset = reqwest::get(format!("https://arweave.net/tx/{}/offset", id))
        .await
        .unwrap()
        .json::<Offset>()
        .await
        .unwrap();

    let mut offset_size = atoi::<i64>(offset.offset.as_bytes()).unwrap();
    let mut tx_size = atoi::<i64>(offset.size.as_bytes()).unwrap();
    let chunk_size = 256 * 1024;

    let mut b: Vec<u8> = Vec::new();
    while tx_size > 0 {
        println!("offset size {} tx left {} chunk size {}", offset_size, tx_size, chunk_size);
        let resp: Response = reqwest::get(format!("https://arweave.net/chunk/{}", offset_size)).await?; //  TODO ALL THE TRANSACTIONS ARE JUST STORED IN THIS HUGE CHUNK ??????????
        let json_chunk: Chunk = match resp.json::<Chunk>().await {
            Ok(x) => x,
            Err(_e) => {
                panic!("{}", format!("could not find chunk for offset {}", offset_size))
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