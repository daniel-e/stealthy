extern crate time;

#[derive(Clone, Debug, Copy)]
pub struct GlobalState {
    start_time: time::Timespec
}

impl GlobalState {
    pub fn new() -> GlobalState {
        GlobalState {
            start_time: time::get_time()
        }
    }

    pub fn uptime(&self) -> i64 {
        time::get_time().sec - self.start_time.sec
    }
}
