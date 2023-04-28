pub trait Identity {
    fn get_id(&self) -> String;
}

pub trait Header {
    fn header() -> Vec<&'static str>;
}
