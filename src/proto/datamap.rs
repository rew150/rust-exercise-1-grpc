use serde::{Deserialize, Serialize};

tonic::include_proto!("datamap");

#[derive(Deserialize, Serialize)]
#[serde(remote = "Entry")]
struct SerdeEntry {
    key: String,
    value: i64,
}

#[derive(Deserialize, Serialize)]
pub struct WrappedEntry(#[serde(with = "SerdeEntry")] pub Entry);
