extern crate tui;
extern crate termion;

use tui::Terminal;
use tui::backend::MouseBackend;
use tui::layout::{Rect, Direction, Size, Group};
use tui::style::{Color, Style, Modifier};
use tui::widgets::{Block, Widget, Borders, Tabs, Paragraph, SelectableList};

use termion::input::TermRead;
use termion::event;

use std::io;
use std::cmp;
use std::iter::repeat;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;

// ------------------------------- Data ------------------------------------------------------------

struct App {
    size: Rect,
    tabs: MyTabs,
    input: String,
    input_cursor: bool,
    messages: Vec<String>,
    line_offset: i32,
    selected_contact: usize,
    contacts: Vec<String>,
    options: Vec<String>,
    selected_option: usize,
    options_content: Options,
}

struct MyTabs {
    pub titles: Vec<String>,
    pub selection: usize,
}

struct Options {
    interface: String,
    add_contact: AddContact,
    focus_on_interface: bool,
    focus_on_options: bool,
}

struct AddContact {
    name: String,
    ip: String,
    key: String,
}

impl MyTabs {
    pub fn next(&mut self) {
        self.selection = (self.selection + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.selection > 0 {
            self.selection -= 1;
        } else {
            self.selection = self.titles.len() - 1;
        }
    }
}

fn create_app(r: Rect) -> App {
    App {
        size: r,
        tabs: MyTabs {
            titles: vec![String::from("Messages"), String::from("Options")],
            selection: 0
        },
        input: String::new(),
        input_cursor: true,
        messages: vec![String::new(), String::new()],
        line_offset: 0,
        selected_contact: 0,
        contacts: vec![String::from("a")],
        options: vec![String::from("Set network interface "), String::from("Add contact          ")],
        selected_option: 0,
        options_content: Options {
            interface: String::new(),
            add_contact: AddContact {
                name: String::new(),
                ip: String::new(),
                key: String::new(),
            },
            focus_on_interface: false,
            focus_on_options: true,
        }
    }
}

// ------------------------------- Data end --------------------------------------------------------

fn update_app(app: &mut App, evt: event::Key) -> bool {
    match evt {
        // https://docs.rs/termion/1.5.1/termion/event/enum.Key.html
        event::Key::Left => {
            app.tabs.previous();
        },
        event::Key::Right => {
            app.tabs.next();
        },
        event::Key::Esc => {
            return true;
        },
        _ => {}
    }

    if app.tabs.selection == 0 { // Messages
        match evt {
            event::Key::Char('\n') => {
                append_message(&mut app.messages[app.selected_contact], &app.input);
                app.input.clear();
                app.line_offset = 0;
            },
            event::Key::Down => {
                if app.line_offset < 0 {
                    app.line_offset += 1;
                }
            },
            event::Key::Up => {
                app.line_offset -= 1;
            },
            event::Key::PageDown => {
                app.selected_contact = cmp::min(app.contacts.len() - 1, app.selected_contact + 1)
            }
            event::Key::PageUp => {
                if app.selected_contact > 0 {
                    app.selected_contact -= 1;
                }
            }
            event::Key::Backspace => {
                app.input.pop();
            },
            event::Key::Char(c) => {
                app.input.push(c);
            },
            _ => {}
        }
    } else if app.tabs.selection == 1 { // Options tab is selected
        match evt {
            event::Key::Char('\n') => {
                if app.options_content.focus_on_options { // if Enter is pressed on one of the options
                    app.options_content.focus_on_options = false;
                    app.options_content.focus_on_interface = false;
                    match app.selected_option {
                        0 => { // "Set network interface"
                            app.options_content.focus_on_interface = true;
                        },
                        1 => { // "Add contact"

                        },
                        _ => {}
                    }
                } else if app.options_content.focus_on_interface {
                    app.options_content.focus_on_options = true;
                    app.options_content.focus_on_interface = false;
                }
            },
            event::Key::Char(c) => {
                if app.options_content.focus_on_interface {
                    app.options_content.interface.push(c);
                }
            },
            event::Key::Backspace => {
                if app.options_content.focus_on_interface {
                    app.options_content.interface.pop();
                }
            },
            event::Key::Down => {
                if app.options_content.focus_on_options {
                    app.options_content.focus_on_interface = false;
                    if  app.selected_option < app.options.len() - 1 {
                        app.selected_option += 1;
                    }
                }
            },
            event::Key::Up => {
                if app.options_content.focus_on_options {
                    if app.selected_option > 0 {
                        app.selected_option -= 1;
                    }
                }
            }
            _ => {}
        }
    }
    app.input_cursor = true;
    false
}

fn keyboard_input() -> Receiver<event::Key> {
    let (tx, rx) = channel();
    thread::spawn(move || {
        let stdin = io::stdin();
        for c in stdin.keys() {
            tx.send(c.unwrap());
        }
    });
    rx
}


fn main() {
    let backend = MouseBackend::new().unwrap();
    let mut term = Terminal::new(backend).unwrap();

    term.clear().unwrap();
    term.hide_cursor().unwrap();

    let mut app = create_app(term.size().unwrap());

    draw(&mut term, &mut app);

    let key_rx = keyboard_input();

    loop {
        match key_rx.recv_timeout(Duration::from_millis(500)) {
            Ok(c) => {
                if update_app(&mut app, c) {
                    break;
                }
            },
            _ => { app.input_cursor = !app.input_cursor; }
        }
        draw(&mut term, &mut app);
    }

    term.clear();
    term.show_cursor().unwrap();
    println!("Goodbye!\r");
}

fn append_message(buf: &mut String, msg: &str) {
    // limit to maximum of 1000 lines
    loop {
        let n = buf.split("\n").count();
        if n < 1000 {
            break;
        }
        let s: String = buf.chars().skip_while(|x| *x != '\n').skip(1).collect();
        buf.clear();
        buf.push_str(&s);
    }
    if buf.len() > 0 {
        buf.push('\n');
    }
    buf.push_str(msg);
}

fn draw(t: &mut Terminal<MouseBackend>, app: &mut App) {
    let app_size = app.size.clone();

    Group::default()
        .direction(Direction::Vertical)
        .margin(0)
        .sizes(&[Size::Fixed(3), Size::Min(0), Size::Fixed(3)])
        .render(t, &app_size, |t, chunks| {

            Tabs::default()
                .block(Block::default().borders(Borders::ALL))
                .titles(&app.tabs.titles)
                .select(app.tabs.selection)
                .style(Style::default().fg(Color::Cyan))
                .highlight_style(Style::default().fg(Color::Yellow))
                .render(t, &chunks[0]);

            match app.tabs.selection {
                0 => { add_messages_widget(t, &chunks[1], &chunks[2], app); }
                1 => { add_options_widget(t, &chunks[1], app); }
                _ => {}
            }
        });
    t.draw().unwrap()
}

fn number_of_lines_in_window(buf: &String, w: u16, h: u16, skp: usize) -> u16 {
    let lines = buf.split("\n").collect::<Vec<_>>();
    let mut c = 0;
    let mut lc = 0;
    for l in lines.iter().skip(skp) {
        lc += 1;
        let mut n = l.chars().count(); // number of characters for current line
        if lc < lines.len() as u16{
            n += 1;  // if not last line add n++ because it contains a newline
        }
        while n > 0 {
            c += 1;
            n -= cmp::min(n, w as usize);
        }
    }
    c
}

fn add_messages_widget(t: &mut Terminal<MouseBackend>, chunk1: &Rect, chunk2: &Rect, app: &mut App) {
    let siz = t.size().unwrap();

    let h = siz.height - 8;
    let w = siz.width - 2;
    let k = app.messages[app.selected_contact].split("\n").count() as u16;
    let mut s: i32 = 0;
    if k > h {
        s = (k - h) as i32;
        while number_of_lines_in_window(&app.messages[app.selected_contact], w, h, s as usize) > h {
            s += 1;
        }
    }
    s += app.line_offset;
    if s < 0 {
        app.line_offset -= s;
        s = 0;
    }

    Group::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .sizes(&[Size::Min(0), Size::Fixed(15)])
        .render(t, chunk1, |t, chunks| {
            // Messages
            let mut title = String::from(format!(" {} ", app.contacts[app.selected_contact]));
            Paragraph::default()
                .style(Style::default())
                .block(Block::default().borders(Borders::ALL).title(&title))
                .text(&app.messages[app.selected_contact])
                .scroll(s as u16)
                .wrap(true)
                .render(t, &chunks[0]);

            SelectableList::default()
                .block(Block::default().title(" Contacts ").borders(Borders::ALL))
                .items(&app.contacts)
                .select(app.selected_contact)
                .highlight_style(Style::default().fg(Color::Yellow).modifier(Modifier::Bold))
                .highlight_symbol("▶")
                .render(t, &chunks[1]);
        });


    input_field(t, chunk2, app.input_cursor, &app.input, siz.width - 2, true);
}

fn input_field(t: &mut Terminal<MouseBackend>, chunk: &Rect, show_cursor: bool, text: &str, length: u16, has_focus: bool) {
    let mut s = String::from(text);
    if show_cursor && has_focus {
        s.push('▄');
    } else {
        s.push(' ');
    }
    let n = length as usize;
    let slen = s.chars().count();
    if slen > n {
        s = s.chars().skip(slen - n).collect();
    }
    let c = if has_focus { Color::DarkGray } else { Color::Black };
    Paragraph::default()
        .style(Style::default().fg(Color::Yellow).bg(c))
        .block(Block::default().borders(Borders::ALL).title(" Your message "))
        .text(&s)
        .render(t, chunk);
}

fn add_options_widget(t: &mut Terminal<MouseBackend>, chunk: &Rect, app: &mut App) {
    Group::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .sizes(&[Size::Fixed(25), Size::Min(0)])
        .render(t, chunk, |t, chunks| {
            SelectableList::default()
                .block(Block::default().title(" Options ").borders(Borders::ALL))
                .items(&app.options)
                .select(app.selected_option)
                .highlight_style(Style::default().fg(Color::Yellow).bg(Color::Gray).modifier(Modifier::Bold))
                .render(t, &chunks[0]);
            Block::default().borders(Borders::ALL).render(t, &chunks[1]);
            match app.selected_option {
                0 => { add_network_interface_widget(t, &chunks[1], app); },
                1 => { add_contact_widget(t, &chunks[1]); },
                _ => {}
            }
        });
}

fn add_network_interface_widget(t: &mut Terminal<MouseBackend>, chunk: &Rect, app: &mut App) {
    Group::default()
        .direction(Direction::Vertical)
        .margin(2)
        .sizes(&[Size::Fixed(1), Size::Min(0)])
        .render(t, chunk, |t, chunks| {
            add_textfield("Interface:",12, 10, t, &chunks[0], app.options_content.focus_on_interface, &app.options_content.interface, app.input_cursor);
            add_placeholder(t, &chunks[1]);
        });
}

fn add_placeholder(t: &mut Terminal<MouseBackend>, chunk: &Rect) {
    Paragraph::default()
        .render(t, chunk);
}

fn add_contact_widget(t: &mut Terminal<MouseBackend>, chunk: &Rect) {
    Group::default()
        .direction(Direction::Vertical)
        .margin(2)
        .sizes(&[Size::Fixed(1), Size::Fixed(1), Size::Fixed(1)])
        .render(t, chunk, |t, chunks| {
            //add_textfield("Name:", 8, 15, t, &chunks[0]);
            //add_textfield("IP:", 8,15, t, &chunks[1]);
            //add_textfield("Key:", 8,32, t, &chunks[2]);
        });
}

fn add_textfield(label: &str, label_length:usize, input_length: usize, t: &mut Terminal<MouseBackend>, chunk: &Rect, has_focus: bool, value: &String, show_cursor: bool) {
    Group::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .sizes(&[Size::Fixed(label_length as u16), Size::Fixed(input_length as u16), Size::Min(0)])
        .render(t, chunk, |t, chunks| {
            Paragraph::default()
                .style(Style::default().fg(Color::Yellow))
                .text(label)
                .render(t, &chunks[0]);
            input_line(t, &chunks[1], show_cursor, value, input_length as u16, has_focus);
            add_placeholder(t, &chunks[2]);
        });
}

fn input_line(t: &mut Terminal<MouseBackend>, chunk: &Rect, show_cursor: bool, text: &str, length: u16, has_focus: bool) {
    let mut s = String::from(text);
    if show_cursor && has_focus {
        s.push('▄');
    } else {
        s.push(' ');
    }
    let n = length as usize;
    let slen = s.chars().count();
    if slen > n {
        s = s.chars().skip(slen - n).collect();
    }
    if has_focus {
        Paragraph::default()
            .style(Style::default().fg(Color::Yellow).bg(Color::DarkGray))
            .text(&s)
            .render(t, chunk);
    } else {
        let ds = if text.len() < length as usize {
            let n = length as usize - text.len();
            String::from(text) + &repeat('_').take(n).collect::<String>()
        } else {
            text.chars().take(length as usize).collect::<>()
        };
        Paragraph::default()
            .style(Style::default().fg(Color::Yellow))
            .text(&ds)
            .render(t, chunk);
    }
}
