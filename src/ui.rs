use term::color;
use ncurses::*;
use std::sync::Arc;
use std::sync::Mutex;

use crate::console::Color;

fn map_color(c: Color) -> color::Color {
    match c {
        Color::BrightGreen => color::BRIGHT_GREEN,
        Color::White => color::WHITE,
        Color::Yellow => color::YELLOW,
        Color::Blue => color::BLUE,
        Color::Red => color::RED,
        Color::Green => color::GREEN,
        Color::BrightRed => color::BRIGHT_RED,
        Color::BrightYellow => color::BRIGHT_YELLOW
    }
}

pub struct Screen {

}

impl Screen {
    pub fn new() -> Screen {
        Screen { }
    }
}

pub enum ControlType {
    ArrowUp,
    ArrowDown
}

pub enum UserInput {
    Line(String),
    Control(ControlType)
}

struct WindowWrapper {
    pub win: WINDOW
}

unsafe impl Send for WindowWrapper { }


pub struct NcursesOut {
    win1: WindowWrapper,
    win2: WindowWrapper,
    scroll_offset: i32,
    max_x: i32,
    max_y: i32,
}

pub struct NcursesIn {
    maxx: i32,
    maxy: i32,
    scr: Arc<Mutex<Screen>>,
}

static COLOR_WHITE_ON_BKGD: i16 = 1;
static COLOR_YELLOW_ON_BKGD: i16 = 2;
static COLOR_RED_ON_BKGD: i16 = 3;
static COLOR_BLUE_ON_BKGD: i16 = 4;
static COLOR_GREEN_ON_BKGD: i16 = 5;
static COLOR_WHITE_ON_RED: i16 = 6;

const K_BACKSPACE: i32 = 127;

const BUFFER_LINES: i32 = 1000;

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
        unsafe {
            getmaxyx(stdscr, &mut max_y, &mut max_x)
        }

        let w1 = WindowWrapper { win: newpad(BUFFER_LINES, max_x) };
        let w2 = WindowWrapper { win: newwin(2, max_x, max_y - 1 - 1, 0) };

        for x in 0..max_x {
            mvwaddch(w2.win, 0, x, '=' as chtype);
        }
        wrefresh(w2.win);
        scrollok(w1.win, true);
        prefresh(w1.win, 0, 0, 0, 0, max_y - 3, max_x);

        NcursesOut {
            win1: w1,
            win2: w2,
            scroll_offset: 0,
            max_y: max_y,
            max_x: max_x,
        }
    }

    pub fn close(&self) {
        endwin();
    }

    pub fn println(&mut self, s: String, color: Color) {

        let attr = match map_color(color) {
            color::YELLOW       => COLOR_PAIR(COLOR_YELLOW_ON_BKGD),
            color::RED          => COLOR_PAIR(COLOR_RED_ON_BKGD),
            color::BLUE         => COLOR_PAIR(COLOR_BLUE_ON_BKGD),
            color::BRIGHT_RED   => COLOR_PAIR(COLOR_RED_ON_BKGD),
            color::GREEN        => COLOR_PAIR(COLOR_GREEN_ON_BKGD),
            color::BRIGHT_GREEN => COLOR_PAIR(COLOR_GREEN_ON_BKGD), // TODO bright
            _                   => COLOR_PAIR(COLOR_WHITE_ON_BKGD)
        };
        let (y, x) = self.pos();
        waddstr(self.win1.win, "\n");
        wattron(self.win1.win, attr as u64);
        waddstr(self.win1.win, &s);
        wattroff(self.win1.win, attr as u64);
        self.jump_to_cursor();
        mv(y, x);
        prefresh(self.win1.win, self.scroll_offset, 0, 0, 0, self.max_y - 3, self.max_x);
        wrefresh(self.win2.win);
    }

    pub fn scroll_up(&mut self) {
        self.scroll_n(-1);
    }

    pub fn scroll_down(&mut self) {
        let mut x = 0;
        let mut y = 0;
        getyx(self.win1.win, &mut y, &mut x);
        if y - self.scroll_offset > self.max_y - 3 {
            self.scroll_n(1);
        }
    }

    // ========================================================================================

    fn scroll_n(&mut self, n: i32) {
        if self.scroll_offset + n >= 0 && self.scroll_offset + n + self.max_y - 2 <= BUFFER_LINES {
            self.scroll_offset += n;
            prefresh(self.win1.win, self.scroll_offset, 0, 0, 0, self.max_y - 3, self.max_x);
        }
    }

    fn jump_to_cursor(&mut self) {
        let mut cx = 0;
        let mut cy = 0;
        getyx(self.win1.win, &mut cy, &mut cx);
        if cy > self.max_y - 3 {
            self.scroll_offset = cy - (self.max_y - 3);
            prefresh(self.win1.win, self.scroll_offset, 0, 0, 0, self.max_y - 3, self.max_x);
        }
    }

    fn pos(&self) -> (i32, i32) {
        let mut x = 0;
        let mut y = 0;
        unsafe {
            getyx(stdscr, &mut y, &mut x);
        }
        //getyx(stdscr(), &mut y, &mut x);
        (y, x)
    }
}

impl NcursesIn {

    pub fn new(scr: Arc<Mutex<Screen>>) -> NcursesIn {

        init_pair(COLOR_WHITE_ON_RED, COLOR_WHITE, COLOR_RED);

        let mut max_x = 0;
        let mut max_y = 0;
        unsafe {
            getmaxyx(stdscr, &mut max_y, &mut max_x);
        }

        NcursesIn {
            maxx: max_x,
            maxy: max_y,
            scr: scr,
        }
    }

    pub fn read_line(&self) -> Option<UserInput> {

        {
            let _scr = self.scr.lock().expect("Mutex lock failed.");
            self.clear_input_line();
            mv(self.maxy - 1, 0);
            refresh();
        }

        let mut buf: Vec<u8> = Vec::new();
        let mut state = 0;

        loop {
            {
                let _scr = self.scr.lock().expect("Mutex lock failed.");
                refresh();
            }
            let c = getch();
            let _scr = self.scr.lock().expect("Mutex lock failed.");

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

                K_BACKSPACE => {
                    // Remove character by overwriting with whitespace.
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

                // If no special key add it to the input buffer.
                _ => {
                    addch(c as chtype); // write it to the screen
                    buf.push(c as u8);  // add it to the buffer
                }
            }

            // If end of line has been reached ...
            if self.x() == self.maxx - 1 {
                self.clear_input_line();
                mv(self.maxy - 1, 0);
            }
        }
    }

    // ============================================================================================

    fn clear_input_line(&self) {

        for x in 0..self.maxx {
            mv(self.maxy - 1, x);
            addch(' ' as chtype);
        }
    }

    fn x(&self) -> i32 {
        let mut x = 0;
        let mut y = 0;
        unsafe {
            getyx(stdscr, &mut y, &mut x);
        }
        x
    }

}


