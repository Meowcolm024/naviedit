use std::io::Stdout;
use termion::{event::Key, raw::RawTerminal};

use super::{
    lib::{read_file, write_file, CurMov, Mode, Row},
    state::State,
    view::View,
};

pub struct Editor<'a> {
    mode: Mode,
    buffer: Vec<String>,
    cmd: Vec<char>,
    focus: (usize, usize),
    name: Option<&'a str>,
    view: View<'a>,
    state: State,
}

impl Editor<'_> {
    pub fn new<'a>(name: Option<&'a str>, stdout: &'a mut RawTerminal<Stdout>) -> Editor<'a> {
        let size = termion::terminal_size().unwrap();
        let mut buffer = Vec::new();
        if let Some(n) = name {
            read_file(n, &mut buffer).unwrap_or_else(|_| {
                buffer.clear();
                buffer.push(String::new())
            });
        } else {
            buffer.push(String::new());
        }
        Editor {
            mode: Mode::Base,
            buffer,
            cmd: Vec::new(),
            focus: (0, 0),
            name,
            view: View::new(stdout, (size.0 as usize, size.1 as usize)),
            state: State::new(),
        }
    }

    pub fn init(&mut self) {
        self.clear();
        self.render();
    }

    fn clear(&mut self) {
        self.view.clear().unwrap();
    }

    pub fn render(&mut self) {
        self.view
            .render(self.mode, &self.focus, &self.buffer, &self.cmd)
            .unwrap();
    }

    fn update_focus(&mut self, dir: CurMov) {
        match dir {
            CurMov::Up => {
                if self.focus.1 > 0 {
                    self.focus.1 -= 1;
                    if self.focus.0 >= self.buffer[self.focus.1].len() {
                        self.focus.0 = self.buffer[self.focus.1].len();
                    }
                }
            }
            CurMov::Down => {
                if self.focus.1 < self.buffer.len() {
                    self.focus.1 += 1;
                    if self.focus.0 >= self.buffer[self.focus.1].len() {
                        self.focus.0 = self.buffer[self.focus.1].len();
                    }
                }
            }
            CurMov::Left => {
                if self.focus.0 > 0 {
                    self.focus.0 -= 1;
                }
            }
            CurMov::Right => {
                if self.focus.0 < self.buffer[self.focus.1].len() {
                    self.focus.0 += 1;
                }
            }
            CurMov::Goto(x, y) => {
                assert!(y < self.buffer.len() && x < self.buffer[y].len());
                self.focus.0 = x;
                self.focus.1 = y;
            }
        }
        self.view.focus_to_cursor(self.focus);
    }

    pub fn key_handle(&mut self, key: &Key) {
        match self.mode {
            Mode::Base => self.base_mode(key),
            Mode::Insert => {}
            Mode::Command => self.command_mode(key),
        }
    }

    fn base_mode(&mut self, key: &Key) {
        match key {
            // Key::Char('i') => self.mode = Mode::Insert,
            Key::Char(':') => self.mode = Mode::Command,
            Key::Up => self.update_focus(CurMov::Up),
            Key::Down => self.update_focus(CurMov::Down),
            Key::Left => self.update_focus(CurMov::Left),
            Key::Right => self.update_focus(CurMov::Right),
            _ => (),
        }
    }

    /*
        fn insert_mode(&mut self, key: &Key) {
            match key {
                Key::Esc => self.mode = Mode::Base,
                Key::Up => self.update_cursor(CurMov::Up),
                Key::Down => self.update_cursor(CurMov::Down),
                Key::Left => self.update_cursor(CurMov::Left),
                Key::Right => self.update_cursor(CurMov::Right),
                Key::Delete | Key::Backspace => {
                    if self.cursor.0 > 1 {
                        if self.cursor.0 - 1 == self.indent {
                            self.indent -= 1
                        }
                        self.buffer[self.cursor.1 - 1]
                            .text
                            .replace_range(self.cursor.0 - 2..self.cursor.0 - 1, "");
                        self.buffer[self.cursor.1 - 1].len -= 1;
                        self.update_cursor(CurMov::Left);
                    } else if self.cursor.0 == 1 && self.cursor.1 > 1 {
                        let cur = self.buffer[self.cursor.1 - 1].text.clone();
                        self.buffer[self.cursor.1 - 2].text += cur.as_str();
                        self.buffer[self.cursor.1 - 2].len += cur.len();
                        self.buffer.remove(self.cursor.1 - 1);
                        self.update_cursor(CurMov::Goto(
                            self.buffer[self.cursor.1 - 2].len - cur.len() + 1,
                            self.cursor.1 - 1,
                        ));
                        self.full = true;
                    }
                }
                Key::Char('\n') => {
                    if self.cursor.0 == self.buffer[self.cursor.1 - 1].len + 1 {
                        self.buffer.insert(
                            self.cursor.1,
                            Row {
                                text: String::from(" ").repeat(self.indent),
                                len: self.indent,
                            },
                        );
                        self.update_cursor(CurMov::Goto(1 + self.indent, self.cursor.1 + 1));
                    } else {
                        let row = self.buffer[self.cursor.1 - 1].clone();
                        self.buffer[self.cursor.1 - 1]
                            .text
                            .truncate(self.cursor.0 - 1);
                        self.buffer[self.cursor.1 - 1].len = self.cursor.0 - 1;

                        let new_row = Row {
                            text: String::from(" ").repeat(self.indent)
                                + &row.text[self.cursor.0 - 1..row.len],
                            len: row.len + 1 - self.cursor.0 + self.indent,
                        };
                        self.buffer.insert(self.cursor.1, new_row);
                        self.update_cursor(CurMov::Goto(1 + self.indent, self.cursor.1 + 1));
                        self.full = true;
                    }
                }
                Key::Char(c) => {
                    let p = match (self.cursor.0 - 1 == self.indent, *c == '\t') {
                        (true, true) => {
                            self.indent += 1;
                            ' '
                        }
                        (false, true) => ' ', // ignoring tab here
                        _ => *c,
                    };
                    self.buffer[self.cursor.1 - 1]
                        .text
                        .insert(self.cursor.0 - 1, p);
                    self.buffer[self.cursor.1 - 1].len += 1;
                    self.update_cursor(CurMov::Right);
                }
                _ => (),
            }
        }
    */

    fn command_mode(&mut self, key: &Key) {
        match key {
            Key::Char('\n') => {
                let cmd_str: String = self.cmd.iter().collect();
                let mut cmd: Vec<&str> = cmd_str.split(' ').rev().collect();
                if let Some(x) = cmd.pop() {
                    match x {
                        "q" => {
                            self.clear();
                            std::process::exit(0);
                        }
                        "w" => {
                            if let Some(file) = cmd.pop() {
                                write_file(file, &self.buffer).unwrap();
                            } else if let Some(file) = self.name {
                                write_file(file, &self.buffer).unwrap();
                            }
                        }
                        _ => (),
                    }
                }
                self.cmd.clear();
                self.mode = Mode::Base;
            }
            Key::Char(c) => self.cmd.push(*c),
            Key::Backspace | Key::Delete => {
                self.cmd.pop();
            }
            Key::Esc => {
                self.cmd.clear();
                self.mode = Mode::Base
            }
            _ => (),
        }
    }
}
