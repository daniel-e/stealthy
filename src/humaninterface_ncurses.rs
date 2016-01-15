#[cfg(feature="usencurses")]
extern crate term;
#[cfg(feature="usencurses")]
extern crate ncurses;

#[cfg(feature="usencurses")]
use term::color;
#[cfg(feature="usencurses")]
use self::ncurses::*;

#[cfg(feature="usencurses")]
use humaninterface::{Input, Output, UserInput, ControlType};
#[cfg(feature="usencurses")]
use callbacks::Callbacks;

#[cfg(feature="usencurses")]
struct WindowWrapper {
    pub win: WINDOW
}

#[cfg(feature="usencurses")]
unsafe impl Send for WindowWrapper { }


#[cfg(feature="usencurses")]
pub struct NcursesOut {
    win1: WindowWrapper,
    win2: WindowWrapper,
    scroll_offset : i32,
}

#[cfg(feature="usencurses")]
pub struct NcursesIn {
    maxx: i32,
    maxy: i32,
}

#[cfg(feature="usencurses")]
static COLOR_WHITE_ON_BKGD: i16 = 1;
#[cfg(feature="usencurses")]
static COLOR_YELLOW_ON_BKGD: i16 = 2;
#[cfg(feature="usencurses")]
static COLOR_RED_ON_BKGD: i16 = 3;
#[cfg(feature="usencurses")]
static COLOR_BLUE_ON_BKGD: i16 = 4;
#[cfg(feature="usencurses")]
static COLOR_GREEN_ON_BKGD: i16 = 5;

#[cfg(feature="usencurses")]
impl NcursesOut {

    pub fn new() -> NcursesOut {

        setlocale(LcCategory::all, "");
        initscr();
        start_color();
        use_default_colors();
        clear();
        noecho();
        refresh();

        init_pair(COLOR_WHITE_ON_BKGD, COLOR_WHITE, -1);
        init_pair(COLOR_YELLOW_ON_BKGD, COLOR_YELLOW, -1);
        init_pair(COLOR_RED_ON_BKGD, COLOR_RED, -1);
        init_pair(COLOR_BLUE_ON_BKGD, COLOR_BLUE, -1);
        init_pair(COLOR_GREEN_ON_BKGD, COLOR_GREEN, -1);

        let mut max_x = 0;
        let mut max_y = 0;
        // the the maximum number of rows and columns
        // max_y - 1 is the last line on the screen
        getmaxyx(stdscr, &mut max_y, &mut max_x);

        let w1 = WindowWrapper { win: newwin(max_y - 2, max_x, 0, 0) };
        let w2 = WindowWrapper { win: newwin(2, max_x, max_y - 1 - 1, 0) };

        wbkgd(w1.win, ' ' as chtype | COLOR_PAIR(1) as chtype);
        for x in 0..max_x {
            mvwaddch(w2.win, 0, x, '=' as chtype);
        }
        wrefresh(w2.win);
        wrefresh(w1.win);
        scrollok(w1.win, true);

        NcursesOut {
            win1: w1,
            win2: w2,
            scroll_offset: 0
        }
    }

    fn scroll_n(&mut self, n: i32) {
        self.scroll_offset += n;
        wscrl(self.win1.win, n);
        wrefresh(self.win1.win);
    }

    fn pos(&self) -> (i32, i32) {
        let mut x = 0;
        let mut y = 0;
        getyx(stdscr, &mut y, &mut x);
        (y, x)
    }
}

#[cfg(feature="usencurses")]
impl Output for NcursesOut {

    fn close(&self) {
        endwin();
    }

    fn println(&mut self, s: String, color: color::Color) {

        let n = -self.scroll_offset;
        self.scroll_n(n);

        let attr = match color {
            color::YELLOW       => COLOR_PAIR(COLOR_YELLOW_ON_BKGD),
            color::RED          => COLOR_PAIR(COLOR_RED_ON_BKGD),
            color::BLUE         => COLOR_PAIR(COLOR_BLUE_ON_BKGD),
            color::BRIGHT_RED   => COLOR_PAIR(COLOR_RED_ON_BKGD),
            color::GREEN        => COLOR_PAIR(COLOR_GREEN_ON_BKGD),
            color::BRIGHT_GREEN => COLOR_PAIR(COLOR_GREEN_ON_BKGD), // TODO bright
            _                   => COLOR_PAIR(COLOR_WHITE_ON_BKGD) 
        };
        let (y, x) = self.pos();
        wattron(self.win1.win, attr as i32);
        waddstr(self.win1.win, "\n");
        waddstr(self.win1.win, &s);
        wattroff(self.win1.win, attr as i32);
        mv(y, x);
        wrefresh(self.win1.win);
        wrefresh(self.win2.win);
    }

    fn scroll_up(&mut self) {
        self.scroll_n(1);
    }

    fn scroll_down(&mut self) {
        self.scroll_n(-1);
    }

}

#[cfg(feature="usencurses")]
impl Callbacks for NcursesOut { }

#[cfg(feature="usencurses")]
impl NcursesIn {

    pub fn new() -> NcursesIn {

        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(stdscr, &mut max_y, &mut max_x);

        NcursesIn {
            maxx: max_x,
            maxy: max_y,
        }
    }
    fn clear_input_line(&self) {

        for x in 0..self.maxx {
            mv(self.maxy - 1, x);
            addch(' ' as chtype);
        }
    }

    fn x(&self) -> i32 {
        let mut x = 0;
        let mut y = 0;
        getyx(stdscr, &mut y, &mut x);
        x
    }

}

#[cfg(feature="usencurses")]
impl Input for NcursesIn {

    fn read_line(&self) -> Option<UserInput> {

        let mut buf: Vec<u8> = Vec::new();

        self.clear_input_line();
        mv(self.maxy - 1, 0);
        //addch('>' as chtype);
        //addch('>' as chtype);
        //addch(' ' as chtype);
        refresh();

        let mut state = 0;

        loop {
            refresh();
            let c = getch();

            if state == 2 {
                state = 0;
                if c == 65 {
                    return Some(UserInput::Control(ControlType::ArrowUp));
                } else if c == 66 {
                    return Some(UserInput::Control(ControlType::ArrowDown));
                } else {
                    continue;
                }
            }
            if state == 1 {
                if c == 91 {
                    state = 2;
                    continue;
                } else {
                    state = 0;
                    continue;
                }
            }

            match c as i32 {
                27 => {
                    state = 1;
                }

                10 => { // TODO constant for enter
                    let s = String::from_utf8(buf.clone());
                    match s {
                        Ok(val) => { return Some(UserInput::Line(val)); }
                        _ => { } // TODO
                    }
                }

                8 => { // TODO constant for ctrl h
                }

                4 => { // TODO constant for ctrl d
                    return None;
                }

                127 => { // TODO constant for backspace
                    if self.x() > 0 {
                        mv(self.maxy - 1, self.x() - 1);
                        addch(' ' as chtype);
                        mv(self.maxy - 1, self.x() - 1);
                    }

                    // Remove the correct amount of UTF-8 characters.
                    match String::from_utf8(buf.clone()) {
                        Ok(mut val) => {
                            val.pop();
                            buf.pop();
                            for _ in 0.. (buf.len() - val.len()) {
                                buf.pop();
                            }
                        }
                        _ => {}
                    }
                }

                _ => {
                    addch(c as chtype);
                    buf.push(c as u8);
                }
            }

            if self.x() == self.maxx - 1 {
                self.clear_input_line();
                mv(self.maxy - 1, 0);
            }
        }
    }
}


