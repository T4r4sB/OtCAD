use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

#[derive(Default)]
pub struct WideStringManager {
    memory_w: Vec<Vec<u16>>,
}

impl WideStringManager {
    pub fn new() -> Self {
        Default::default()
    }

    fn from_str_impl(str: &str) -> Vec<u16> {
        return OsStr::new(str)
            .encode_wide()
            .chain(Some(0).into_iter())
            .collect();
    }

    pub fn from_str(&mut self, str: &str) -> *const u16 {
        self.memory_w.push(Self::from_str_impl(str));
        return self.memory_w.last().unwrap().as_ptr();
    }
}
