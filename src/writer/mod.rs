use prost::Message;

pub mod flle_writer;
pub mod formatters;
#[cfg(feature = "kafka")]
pub mod kafka_writer;
pub mod persistance;
pub mod sink;

pub fn encode_message<T: Message>(message: &T) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    message
        .encode(&mut buf)
        .map(|_| buf)
        .map_err(|e| format!("Error encoding message: {:?}", e))
}
