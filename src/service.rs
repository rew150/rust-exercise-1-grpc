use std::collections::HashMap;
use std::sync::{Arc};
use tokio::join;
use tokio::sync::{RwLock};

use tonic::Code;

use crate::jsonfile;
use crate::proto::datamap::data_map_server::{DataMap, DataMapServer};
use crate::proto::datamap;

#[derive(Debug)]
pub struct DataMapService {
    file_loc: String,
    map: Arc<RwLock<HashMap<String, i64>>>,
    freqs: Arc<RwLock<HashMap<String, u32>>>,
}

impl DataMapService {
    fn new(file_loc: String, map: Arc<RwLock<HashMap<String, i64>>>) -> Self {
        DataMapService {
            file_loc,
            map,
            freqs: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[tonic::async_trait]
impl DataMap for DataMapService {
    async fn get(&self,request:tonic::Request<datamap::GetRequest>,) -> Result<tonic::Response<datamap::GetResponse>,tonic::Status> {
        let key = request.into_inner().key;
        let (map, mut freqs) = join!(self.map.read(), self.freqs.write());
        map.get(&key)
            .map(|val| {
                if let Some(fk) = freqs.get_mut(&key) {
                    *fk += 1;
                } else {
                    freqs.insert(key, 1);
                }
                res(datamap::GetResponse { value: *val })
            }).ok_or(key_not_found())
    }

    async fn create(&self, request:tonic::Request<datamap::CreateRequest>) -> Result<tonic::Response<datamap::CreateResponse>,tonic::Status> {
        let req = request.into_inner();
        let key = req.key;
        let val = req.value;
        let mut map = self.map.write().await;
        if map.contains_key(&key) {
            Err(conflict())
        } else {
            map.insert(key, val);
            Ok(res(datamap::CreateResponse {}))
        }
    }

    async fn update(&self,request:tonic::Request<datamap::UpdateRequest>,) -> Result<tonic::Response<datamap::UpdateResponse>,tonic::Status> {
        let req = request.into_inner();
        let key = req.key;
        let new_val = req.value;
        let mut map = self.map.write().await;
        map.get_mut(&key)
            .map(|val| {
                *val = new_val;
                res(datamap::UpdateResponse {})
            }).ok_or(key_not_found())
    }


    async fn drop(&self,request:tonic::Request<datamap::DropRequest>,) -> Result<tonic::Response<datamap::DropResponse>,tonic::Status> {
        let key = request.into_inner().key;
        let mut map = self.map.write().await;
        match map.remove(&key) {
            Some(_) => Ok(res(datamap::DropResponse {})),
            None => Err(key_not_found()),
        }
    }

    async fn get_entries(&self,_:tonic::Request<datamap::GetEntriesRequest> ,) ->  Result<tonic::Response<datamap::GetEntriesResponse> ,tonic::Status> {
        let map = self.map.read().await;
        Ok(res(datamap::GetEntriesResponse {
            entries: map.iter()
                .map(|(key, value)|
                    datamap::Entry {
                        key: key.to_owned(),
                        value: *value,
                    }
                ).collect()
        }))
    }

    async fn flush(&self,_:tonic::Request<datamap::FlushRequest> ,) -> Result<tonic::Response<datamap::FlushResponse> ,tonic::Status> {
        match jsonfile::write_json(&self.file_loc, &self.map).await {
            Some(_) => Ok(res(datamap::FlushResponse {})),
            None => Err(internal_err()),
        }
    }

    async fn get_read_summary(&self,_:tonic::Request<datamap::GetReadSummaryRequest>) -> Result<tonic::Response<datamap::GetReadSummaryResponse>,tonic::Status> {
        let map = self.freqs.read().await;
        Ok(res(
            datamap::GetReadSummaryResponse { freqs: 
                map.iter()
                    .map(|(k, f)|
                        datamap::ReadFrequency {
                            key: k.clone(),
                            freq: f.clone(),
                        }
                    ).collect()
            }
        ))
    }
}

#[inline]
pub fn datamap_server(file_loc: String, map: Arc<RwLock<HashMap<String, i64>>>) -> DataMapServer<DataMapService> {
    DataMapServer::new(DataMapService::new(file_loc, map))
}

#[inline]
fn res<T>(body: T) -> tonic::Response<T> {
    tonic::Response::new(body)
}

#[inline]
fn key_not_found() -> tonic::Status {
    tonic::Status::new(Code::NotFound, "key not found")
}

#[inline]
fn conflict() -> tonic::Status {
    tonic::Status::new(Code::AlreadyExists, "key already exists")
}

#[inline]
fn internal_err() -> tonic::Status {
    tonic::Status::new(Code::Internal, "flushing error")
}
