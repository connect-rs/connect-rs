use crate::ConnectError;
use prost::Message;
use serde::{Serialize, de::DeserializeOwned};

impl<T> crate::ConnectMessageProto for T
where
    T: Message + Default + Send + Sync + 'static,
{
    fn encode_proto(&self) -> Result<Vec<u8>, ConnectError> {
        Ok(self.encode_to_vec())
    }

    fn decode_proto(bytes: &[u8]) -> Result<Self, ConnectError> {
        T::decode(bytes)
            .map_err(|e| ConnectError::invalid_argument(format!("Protobuf decode failed: {e}")))
    }
}

impl<T> crate::ConnectMessageJson for T
where
    T: Message + Default + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    fn encode_json(&self) -> Result<Vec<u8>, ConnectError> {
        serde_json::to_vec(self)
            .map_err(|e| ConnectError::internal(format!("JSON encode failed: {e}")))
    }

    fn decode_json(bytes: &[u8]) -> Result<Self, ConnectError> {
        serde_json::from_slice(bytes)
            .map_err(|e| ConnectError::invalid_argument(format!("Invalid JSON: {e}")))
    }
}
