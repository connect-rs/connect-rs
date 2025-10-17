use crate::ConnectError;
use prost::Message;

impl<T> crate::ConnectMessage for T
where
    T: Message + Default + Send + Sync + 'static,
{
    fn encode_json(&self) -> Result<Vec<u8>, ConnectError> {
        // For now, return unimplemented
        // We'll add proper JSON support later with prost-reflect working
        Err(ConnectError::unimplemented(
            "JSON encoding not yet implemented",
        ))
    }

    fn encode_proto(&self) -> Result<Vec<u8>, ConnectError> {
        Ok(self.encode_to_vec())
    }

    fn decode_json(_bytes: &[u8]) -> Result<Self, ConnectError> {
        Err(ConnectError::unimplemented(
            "JSON decoding not yet implemented",
        ))
    }

    fn decode_proto(bytes: &[u8]) -> Result<Self, ConnectError> {
        T::decode(bytes)
            .map_err(|e| ConnectError::invalid_argument(format!("Protobuf decode failed: {}", e)))
    }
}
