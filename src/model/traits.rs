use super::enums::MessageType;
use super::enums::Payload;

pub trait Identity {
    fn get_id(&self) -> String;
}

pub trait URLResource {
    fn get_url(&self) -> String;
}

pub trait Header {
    fn header() -> Vec<&'static str>;
}

pub trait SetIdentity {
    fn set_id(&mut self, id: String);
}

pub trait PayloadProcessor<T> {
    fn process(&self, payload: Payload<T>) -> Payload<T>;
}

pub trait MessageTransform {
    fn identity(&self) -> String;
    fn transform(&self) -> Result<MessageType, String>;
}
