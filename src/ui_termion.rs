use std::sync::Arc;
use std::sync::Mutex;
use std::io::Write;
use std::io::Stdout;
use std::io::stdout;
use std::cmp::min;
use termion::color::Fg;
use termion::raw::RawTerminal;
use termion::raw::IntoRawMode;

use crate::model::{Item, ItemType, Model};
use crate::model::Source;
use crate::tools::rot13;

static ACK: char = '✔';
static NUMBERS: &str = "➀➁➂➃➄➅➆➇➈➉";

/// Write messages to the terminal.
pub struct TermOut {
    stdout: RawTerminal<Stdout>,
    model: Arc<Mutex<Model>>,
    // The scroll_offset is the amount of "arrows up".
    // When the user scrolls, i.e. the scroll_offset > 0, then a new message should not change the
    // view on the messages in the window. Therefore, adjust_scroll_offset() needs to be called
    // when a new message has been added to the buffer in the model.
    scroll_offset: usize,
    raw_view: bool,
    scramble_view: bool,
}

impl TermOut {

    pub fn new(model: Arc<Mutex<Model>>) -> TermOut {
        TermOut {
            stdout: stdout().into_raw_mode().expect("No raw mode possible."),
            model: model,
            scroll_offset: 0,
            raw_view: false,
            scramble_view: false,
        }.init()
    }

    pub fn close(&mut self) {
        write!(self.stdout, "{}{}{}{}{}",
               termion::clear::All,
               termion::cursor::Goto(1, 1),
               termion::cursor::Show,
               termion::color::Fg(termion::color::Reset),
               termion::color::Bg(termion::color::Reset)
        ).expect("Write error.");
        self.flush();
    }

    /// This method is called after a new message has been received. If the user has scrolled
    /// to some position in the window this method ensures that the content of the window does
    /// not scroll for the new message.
    /// The message is added to the model.
    pub fn adjust_scroll_offset(&mut self, i: Item) {
        if self.scroll_offset > 0 {
            self.increase_scroll_offset(TermOut::split_line(self,&i).len());
        }

        self.redraw();
    }

    pub fn scroll_up(&mut self) {
        self.scroll_up_1();
        self.redraw();
    }

    pub fn scroll_down(&mut self) {
        self.scroll_down_1();
        self.redraw();
    }

    pub fn refresh(&mut self) {
        self.redraw();
    }

    pub fn page_up(&mut self) {
        for _ in 0..self.window_height() {
            self.scroll_up_1();
        }
        self.redraw();
    }

    pub fn page_down(&mut self) {
        for _ in 0..self.window_height() {
            self.scroll_down_1();
        }
        self.redraw();
    }

    pub fn key_end(&mut self) {
        self.scroll_offset = 0;
        self.redraw();
    }

    pub fn toggle_raw_view(&mut self) {
        self.raw_view = !self.raw_view;
        self.increase_scroll_offset(0);
        write!(self.stdout, "{}", termion::clear::All, ).expect("Write error.");
        self.redraw();
    }

    pub fn toggle_scramble_view(&mut self) {
        self.scramble_view = !self.scramble_view;
        self.redraw();
    }

    // ===========================================================================================

    fn init(mut self) -> TermOut {
        write!(self.stdout, "{}{}",
               termion::clear::All,   // clear screen
               termion::cursor::Hide  // hide cursor
        ).expect("Error.");
        self.flush();
        self
    }

    fn flush(&mut self) {
        self.stdout.flush().expect("Flush error.");
    }

    fn increase_scroll_offset(&mut self, n: usize) {
        let model = self.model.lock().unwrap();
        // The number of lines in the window.
        let window_height = self.window_height();
        // The number of lines required to show all messages. One message can consume multiple lines.
        let buffer_lines = self.lines(&model.buf).len();

        if buffer_lines > window_height {
            let max_off = buffer_lines - window_height;
            self.scroll_offset = min(max_off, self.scroll_offset + n);
        }
    }

    fn scroll_up_1(&mut self) {
        self.increase_scroll_offset(1);
    }

    fn scroll_down_1(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    fn draw_window(&mut self) {
        let (maxx, maxy) = TermOut::size();

        for x in 2..maxx {
            write!(self.stdout, "{}─{}─{}─",
                   termion::cursor::Goto(x, 1),
                   termion::cursor::Goto(x, maxy),
                   termion::cursor::Goto(x, maxy - 2),
            ).expect("Error.");
        }
        for y in 2..maxy {
            write!(self.stdout, "{}│{}│",
                   termion::cursor::Goto(1, y),
                   termion::cursor::Goto(maxx, y)
            ).expect("Error.");
        }
        write!(self.stdout,
               "{}┌{}┐{}└{}┘{}├{}┤",
               termion::cursor::Goto(1, 1),
               termion::cursor::Goto(maxx, 1),
               termion::cursor::Goto(1, maxy),
               termion::cursor::Goto(maxx, maxy),
               termion::cursor::Goto(1, maxy - 2),
               termion::cursor::Goto(maxx, maxy - 2)
        ).expect("Error.");
    }

    fn fm_time(&self, i: &Item) -> String {
        time::strftime("%d.%m. %R", &i.tim).unwrap()
    }

    fn progress_bar(&self, pending: usize, total: usize, len: usize) -> String {
        let k = len * (total - pending) / total;
        let s_transmitted = std::iter::repeat('█').take(k).collect::<String>();
        let s_remaining = std::iter::repeat('░').take(len - k).collect::<String>();
        format!("|{}{}| {:.2}%", s_transmitted, s_remaining, (total - pending) as f64 / total as f64 * 100.0)
    }

    fn txt(&self, i: &Item) -> String {
        let msg = i.msg.to_string();

        if self.raw_view {
            return format!("{}", msg);
        }

        // Optionally Scramble
        let maybe_scrambled_msg = if self.scramble_view {
            scramble(&i.msg)
        } else {
            msg.clone()
        };

        // Formatting
        let t = self.fm_time(&i);
        match i.source() {
            Source::Ip(ip) => {
                format!("{} | [{}] {}", t, ip, maybe_scrambled_msg)
            },
            Source::You => {
                match i.typ {
                    ItemType::UploadMessage => {
                        format!("{} | [you] {} {}", t, maybe_scrambled_msg, self.progress_bar(i.pending_acks, i.total_acks, 40))
                    },
                    _ => {
                        format!("{} | [you] {}", t, maybe_scrambled_msg)
                    }
                }
            },
            Source::System => {
                let p = match i.typ {
                    ItemType::Introduction => {
                        String::from("")
                    },
                    _ => {
                        t + " | "
                    }
                };
                format!("{}{}", p, msg)
            },
            Source::Raw => {
                format!("{}", msg)
            }
        }
    }

    fn redraw(&mut self) {
        if !self.raw_view {
            self.draw_window();
        }

        let wx = self.window_x_offset();
        let wy = self.window_y_offset();

        let model = self.model.lock().unwrap();
        let screen_width = self.window_width();
        let screen_height = self.window_height();

        let buffer = self.lines(&model.buf);
        let n = buffer.len();
        let mut p = 0;

        let buf = if n <= screen_height {
            buffer.clone()
        } else {
            // n - screen_height: index for scroll_offset = 0
            p = n - screen_height - self.scroll_offset;
            buffer.iter().skip(p).take(screen_height).cloned().collect()
        };

        // Show messages.
        for (y, line) in buf.iter().enumerate() {
            let t = self.txt(&line); // formatted line
            let m = extend_line_to_screen_width(t, screen_width);

            write_color(&mut self.stdout, line.typ.clone());
            write_at(&mut self.stdout, wx, y + wy, &m);
            if !self.raw_view {
                write_symbol(&mut self.stdout, line, y);
            }
        }

        // Show input field.
        if !self.raw_view {
            write_input_field(&mut self.stdout, model.input.clone());
        }

        // Show scroll status.
        if !self.raw_view && self.scroll_offset > 0 {
            write_scroll_status(&mut self.stdout, p, buffer.len());
        }

        self.stdout.flush().unwrap();
    }

    fn window_x_offset(&self) -> usize {
        if self.raw_view {
            1
        } else {
            2
        }
    }

    fn window_y_offset(&self) -> usize {
        if self.raw_view {
            1
        } else {
            2
        }
    }

    fn window_height(&self) -> usize {
        if self.raw_view {
            TermOut::size().1 as usize
        } else {
            TermOut::size().1 as usize - 4
        }
    }

    fn window_width(&self) -> usize {
        if self.raw_view {
            TermOut::size().0 as usize
        } else {
            TermOut::size().0 as usize - 2
        }
    }

    // ===========================================================================================

    fn size() -> (u16, u16) {
        termion::terminal_size().unwrap()
    }

    fn split_line(&self, s: &Item) -> Vec<Item> {
        // TODO use https://github.com/unicode-rs/unicode-width to estimate the width of UTF-8 characters
        TermOut::remove_symbol(self.txt(s).chars().collect::<Vec<char>>()
            .chunks(self.window_width())
            .map(|x| s.clone().message(x.iter().collect()).raw())
            .collect()
        )
    }

    /// If a message spans multiple lines this method ensures that the symbol is shown only
    /// for the first line.
    fn remove_symbol(mut v: Vec<Item>) -> Vec<Item> {
        for i in v.iter_mut().skip(1) {
            i.id.clear();
        }
        v
    }

    fn lines(&self, buf: &Vec<Item>) -> Vec<Item> {
        buf.iter()
            .map(|v| self.split_line(v))
            .flatten()
            .collect()
    }
}

// -------------------------------------------------------------------------------------------------

fn write_color(o: &mut RawTerminal<Stdout>, typ: ItemType) {
    match typ {
        ItemType::Received => write!(o, "{}", Fg(termion::color::LightGreen)),
        ItemType::Info => write!(o, "{}", Fg(termion::color::Yellow)),
        ItemType::Introduction => write!(o, "{}", Fg(termion::color::Green)),
        ItemType::Error => write!(o, "{}", Fg(termion::color::Red)),
        ItemType::NewFile => write!(o, "{}", Fg(termion::color::LightWhite)),
        ItemType::MyMessage => write!(o, "{}", Fg(termion::color::Green)),
        ItemType::UploadMessage => write!(o, "{}", Fg(termion::color::Green)),
    }.unwrap();
}

fn extend_line_to_screen_width(s: String, screen_width: usize) -> String {
    let mut v = s;
    while v.chars().count() < screen_width {
        v.push(' ');
    }
    v
}

fn symbol_for_item(item: &Item) -> String {
    if item.id.len() == 0 {
        return format!("");
    }

    if item.acks_received >= item.id.len() {
        return format!("{}{}", Fg(termion::color::Green), ACK);
    }

    // pending cannot be zero
    let pending = item.id.len() - item.acks_received;
    let p = min(pending, 10) - 1;
    let v = NUMBERS.chars().collect::<Vec<_>>();

    format!("{}{}", Fg(termion::color::LightYellow), v[p])
}

fn write_symbol(o: &mut RawTerminal<Stdout>, item: &Item, y: usize) {
    let symbol = symbol_for_item(item);
    if symbol.len() == 0 {
        return;
    }
    write!(o, "{}{}{}",
           termion::cursor::Goto(16, y as u16 + 2),
           symbol,
           termion::color::Fg(termion::color::Reset)
    ).expect("Write failed.");
}

fn write_at(o: &mut RawTerminal<Stdout>, x: usize, y: usize, s: &str) {
    write!(o, "{}{}{}{}",
           termion::cursor::Goto(x as u16, y as u16),
           s,
           termion::color::Fg(termion::color::Reset),
           termion::color::Bg(termion::color::Reset)
    ).expect("Error.");
}

fn write_input_field(o: &mut RawTerminal<Stdout>, input: Vec<u8>) {

    let (maxx, maxy) = TermOut::size();
    let input_field_len = maxx - 2 - 1;

    write!(o, "{}", termion::color::Bg(termion::color::Blue)).expect("Error.");
    for x in 2..maxx {
        write!(o, "{} ", termion::cursor::Goto(x, maxy - 1)).expect("Error.");
    }
    let mut s = String::from_utf8(input).unwrap();
    while s.chars().count() > input_field_len as usize {
        s.remove(0);

    }
    s.push('▂');
    write_at(o, 2, maxy as usize - 1, &s);
}

fn write_scroll_status(o: &mut RawTerminal<Stdout>, current: usize, len: usize) {
    let (maxx, _) = TermOut::size();
    let s = format!("line:{}/{}", current, len);
    let x = maxx as usize - s.len();
    write!(o, "{}{}{}{}{}{}",
           termion::cursor::Goto(x as u16, 2),
           termion::color::Bg(termion::color::Red),
           termion::color::Fg(termion::color::LightWhite),
           s,
           termion::color::Bg(termion::color::Reset),
           termion::color::Fg(termion::color::Reset)
    ).expect("Error.");
}

fn scramble(i: &String) -> String {
    i.chars().map(|c| rot13(c)).collect::<String>()
}
