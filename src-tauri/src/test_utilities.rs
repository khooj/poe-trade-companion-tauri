use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Callable {
    m: Arc<Mutex<u64>>,
}

impl Callable {
    pub fn new() -> Self {
        Callable {
            m: Arc::new(Mutex::new(0)),
        }
    }

    pub fn call(&self) {
        *self.m.lock().unwrap() += 1;
    }

    pub fn count(&self) -> u64 {
        *self.m.lock().unwrap()
    }
}
