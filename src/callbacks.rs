extern crate time;

//use term::color;

use humaninterface::Output;

pub trait Callbacks : Output {
/*
    fn err_msg(&mut self, msg: String) {
        let fm = time::strftime("%R", &time::now()).unwrap();
        self.println(format!("{} error: {}", fm, msg), color::BRIGHT_RED);
    }

    fn write_msg(&mut self, s: String) {
        let fm = time::strftime("%R", &time::now()).unwrap();
        self.println(format!("{} {}", fm, s), color::BRIGHT_GREEN);
    }
    */
}


