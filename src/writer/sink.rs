#[async_trait::async_trait]
pub trait Sink<T>: Send + Sync + 'static {
    async fn write(&self, message: T) -> Result<(), String>;
    async fn flush(&self) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum SinkType {
    Kafka,
    ProtobufFile,
    CsvFile,
}

#[derive(Debug, Clone)]
pub enum FormatterType {
    Protobuf,
    Csv,
}
