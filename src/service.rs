use std::collections::HashMap;

use tonic::Code;

use crate::proto::datamap::data_map_server::{DataMap, DataMapServer};
use crate::proto::datamap;

#[derive(Debug, Default)]
pub struct DataMapService {
    // use std::sync::Mutex as recommended by tokio
    map: std::sync::Mutex<HashMap<String, i64>>,
}

#[tonic::async_trait]
impl DataMap for DataMapService {
    async fn get(&self,request:tonic::Request<datamap::GetRequest>,) -> Result<tonic::Response<datamap::GetResponse>,tonic::Status> {
        let key = &request.get_ref().key;
        let map = self.map.lock().expect("Mutex: poisoned lock");
        map.get(key)
            .map(|val|
                res(datamap::GetResponse { value: *val })
            ).ok_or(key_not_found())
    }

    async fn create(&self, request:tonic::Request<datamap::CreateRequest>) -> Result<tonic::Response<datamap::CreateResponse>,tonic::Status> {
        let req = request.get_ref();
        let key = &req.key;
        let val = req.value;
        let mut map = self.map.lock().expect("Mutex: poisoned lock");
        if map.contains_key(key) {
            Err(conflict())
        } else {
            map.insert(key.clone(), val);
            Ok(res(datamap::CreateResponse {}))
        }
    }

    async fn update(&self,request:tonic::Request<datamap::UpdateRequest>,) -> Result<tonic::Response<datamap::UpdateResponse>,tonic::Status> {
        let req = request.get_ref();
        let key = &req.key;
        let new_val = req.value;
        let mut map = self.map.lock().expect("Mutex: poisoned lock");
        map.get_mut(key)
            .map(|val| {
                *val = new_val;
                res(datamap::UpdateResponse {})
            }).ok_or(key_not_found())
    }


    async fn drop(&self,request:tonic::Request<datamap::DropRequest>,) -> Result<tonic::Response<datamap::DropResponse>,tonic::Status> {
        let key = &request.get_ref().key;
        let mut map = self.map.lock().expect("Mutex: poisoned lock");
        match map.remove(key) {
            Some(_) => Ok(res(datamap::DropResponse {})),
            None => Err(key_not_found()),
        }
    }
}

#[inline]
pub fn datamap_server() -> DataMapServer<DataMapService> {
    DataMapServer::new(DataMapService::default())
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
