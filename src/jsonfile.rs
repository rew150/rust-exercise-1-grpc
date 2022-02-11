use std::{fs::{OpenOptions}, collections::HashMap, ops::Deref};
use tokio::sync::RwLock;
use tokio::fs as tokiofs;

use crate::proto::datamap::{WrappedEntry, self};

// use sync read because we are doint one-time init here
pub fn try_read_json_init(file_loc: &str) -> HashMap<String, i64> {
    OpenOptions::new()
        .read(true)
        .open(file_loc)
        .map_err(|_| 
            println!("error open file at {}, ignoring it", file_loc)
        ).ok()
        .and_then(|f|
            serde_json::from_reader::<_, Vec<WrappedEntry>>(f)
                .map_err(|_|
                    println!("error parsing json, ignoring it")
                ).ok()
        ).unwrap_or(Vec::new())
        .into_iter()
        .map(|WrappedEntry(entry)|
            (entry.key, entry.value)
        )
        .collect()
}

pub async fn write_json(file_loc: &str, map: &RwLock<HashMap<String, i64>>) -> Option<()> {
    let data: Vec<WrappedEntry> = map
        .read()
        .await
        .deref()
        .iter()
        .map(|(k, v)| 
            WrappedEntry(datamap::Entry{
                key: k.clone(),
                value: v.clone(),
            })
        ).collect();

    let json_string = serde_json::to_string(&data)
        .map_err(|_|
            println!("error serialization")
        ).ok()?;

    tokiofs::write(file_loc, json_string)
        .await
        .map_err(|_| 
            println!("error writing to file {}", file_loc)
        ).ok()
}
