pub trait Clipboard: std::fmt::Debug {
    fn get_string(&self) -> Option<String>;
    fn put_string(&mut self, text: &str);
}
