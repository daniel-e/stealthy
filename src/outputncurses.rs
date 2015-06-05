extern crate ncurses;
use ncurses::*;

struct Ncurses {
    r_x: i32,
    r_y: i32,
    w_x: i32,
    w_y: i32
}

impl Ncurses {

    fn new() -> Ncurses {
        initscr();
        clear();
        refresh();
        
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(stdscr, &mut max_y, &mut max_x);
        mv(max_y, 0);

        Ncurses {
            r_x: 0,
            r_y: 0,
            w_x: 0,
            w_y: max_y
        }
    }

    fn print(&self, x: i32, y: i32, s: String) {

        mv(y, x);
        printw(&s);
        refresh();
    }

    fn read_line(&self, s: &mut String) -> i32 {
        getch();
        s.push_str("bla");
        1
    }
}

impl Drop for Ncurses {
    fn drop(&mut self) {
        endwin();
    }
}

impl Ncurses {
    /// This function is called when a new message has been received.
    fn new_msg(&self, msg: Message) {

        let ip = msg.get_ip();
        let s  = String::from_utf8(msg.get_payload());
        let fm = time::strftime("%R", &time::now()).unwrap();

        match s {
            Ok(s)  => { tools::println_colored(format!("[{}] {} says: {}", ip, fm, s), term::color::YELLOW); }
            Err(_) => { 
                tools::println_colored(format!("[{}] {} error: could not decode message", ip, fm), term::color::BRIGHT_RED); 
            }
        }
    }

    /// This callback function is called when the receiver has received the
    /// message with the given id.
    ///
    /// Important note: The acknowledge that is received here is the ack on the
    /// network layer which is not protected. An
    /// attacker could drop acknowledges or could fake acknowledges. Therefore,
    /// it is important that acknowledges are handled on a higher layer where
    /// they can be protected via cryptographic mechanisms.
    fn ack_msg(&self, _id: u64) {

        tools::println_colored("ack".to_string(), term::color::BRIGHT_GREEN);
    }
}



