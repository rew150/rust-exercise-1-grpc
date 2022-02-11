use std::collections::HashMap;
use std::sync::{Arc};
use tokio::sync::{RwLock};

use tonic::Code;

use crate::proto::datamap::data_map_server::{DataMap, DataMapServer};
use crate::proto::datamap;

#[derive(Debug)]
pub struct DataMapService {
    map: Arc<RwLock<HashMap<String, i64>>>,
}

impl DataMapService {
    fn new(map: Arc<RwLock<HashMap<String, i64>>>) -> Self {
        DataMapService {
            map,
        }
    }
}

#[tonic::async_trait]
impl DataMap for DataMapService {
    async fn get(&self,request:tonic::Request<datamap::GetRequest>,) -> Result<tonic::Response<datamap::GetResponse>,tonic::Status> {
        let key = request.into_inner().key;
        let map = self.map.read().await;
        map.get(&key)
            .map(|val|
                res(datamap::GetResponse { value: *val })
            ).ok_or(key_not_found())
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


    async fn flush(&self,request:tonic::Request<datamap::FlushRequest> ,) -> Result<tonic::Response<datamap::FlushResponse> ,tonic::Status> {
        todo!()
    }

}

#[inline]
pub fn datamap_server(map: Arc<RwLock<HashMap<String, i64>>>) -> DataMapServer<DataMapService> {
    DataMapServer::new(DataMapService::new(map))
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
