use std::rc::Rc;

#[derive(Default)]
pub struct JobSystem {
  jobs: Vec<Rc<dyn Fn()>>,
}

impl JobSystem {
  pub fn new() -> Self {
    Self {
      jobs: Vec::new(),
    }
  }

  pub fn add_callback(&mut self, callback: Rc<dyn Fn() + 'static>) {
    self.jobs.push(callback);
  }

  pub fn run_all(&mut self) {
    for callback in self.jobs.iter() {
      callback();
    }
    self.jobs.clear();
  }
}
