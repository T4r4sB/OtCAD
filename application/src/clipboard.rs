use std::cell::RefCell;
use std::rc::Rc;

pub trait ClipboardHandler: std::fmt::Debug {
    fn get_string(&self) -> Option<String>;
    fn put_string(&mut self, text: &str);
}

#[derive(Debug, Clone)]
pub struct Clipboard {
    handler: Rc<RefCell<dyn ClipboardHandler>>,
}

impl Clipboard {
    pub fn new(handler: impl ClipboardHandler + 'static) -> Self {
        Self {
            handler: Rc::new(RefCell::new(handler)),
        }
    }
    pub fn get_string(&self) -> Option<String> {
        self.handler.borrow().get_string()
    }
    pub fn put_string(&mut self, text: &str) {
        self.handler.borrow_mut().put_string(text)
    }
}
