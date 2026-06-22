pub trait ResourceProvider {
    fn load(&self, url: &str) -> Option<Vec<u8>>;
}

pub trait EventSink {
    fn navigate(&mut self, url: &str);
    fn set_cursor(&mut self, cursor: &str);
}
