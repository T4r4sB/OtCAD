use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default, Clone)]
pub struct JobSystem {
    jobs: Rc<RefCell<Vec<Rc<dyn Fn()>>>>,
}

impl JobSystem {
    pub fn new() -> Self {
        Self {
            jobs: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn add_callback(&self, callback: Rc<dyn Fn() + 'static>) {
        self.jobs.borrow_mut().push(callback);
    }

    pub fn run_all(&self) -> bool {
        let result = !self.jobs.borrow().is_empty();
        for callback in self.jobs.borrow().iter() {
            callback();
        }
        self.jobs.borrow_mut().clear();
        result
    }
}
