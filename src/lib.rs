use std::io::{BufRead, BufReader, Stdout, Write};
use termion::{event::Key, raw::RawTerminal};

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Insert,  // insert mode
    Base,    // base mode
    Command, // command input mode
}

#[derive(Debug, Clone)]
struct Row {
    text: String,
    len: usize,
}

impl Row {
    pub fn new() -> Row {
        Row {
            text: String::new(),
            len: 0,
        }
    }
}

pub struct Editor<'a> {
    mode: Mode,
    buffer: Vec<Row>,
    cursor: (usize, usize),
    size: (usize, usize),
    stdout: &'a mut RawTerminal<Stdout>,
    full: bool,
    name: Option<String>,
    cmd: Vec<char>,
}

#[derive(Debug, Clone, Copy)]
enum CurMov {
    Up,
    Down,
    Left,
    Right,
    Goto(usize, usize),
}

impl Editor<'_> {
    pub fn new<'a>(name: Option<String>, stdout: &'a mut RawTerminal<Stdout>) -> Editor<'a> {
        let sz = termion::terminal_size().unwrap();
        let mut buffer = Vec::new();
        if let Some(n) = name.clone() {
            read_file(n.as_str(), &mut buffer).unwrap_or_else(|_| {
                buffer.clear();
                buffer.push(Row::new())
            });
        } else {
            buffer.push(Row::new());
        }
        Editor {
            mode: Mode::Base,
            buffer: buffer,
            cursor: (1, 1),
            size: (sz.0 as usize, sz.1 as usize),
            stdout: stdout,
            full: false,
            name: name,
            cmd: Vec::new(),
        }
    }

    pub fn init(&mut self) {
        self.clear();
        self.mode = Mode::Insert;
        self.full = true;
        self.render();
        self.mode = Mode::Base;
        self.render();
    }

    fn clear(&mut self) {
        write!(self.stdout, "{}", termion::clear::All).unwrap();
        self.stdout.flush().unwrap();
    }

    fn render_line(&mut self, row: usize) {
        let (l, r) = match (
            self.cursor.0 < self.size.0,
            self.size.0 > self.buffer[row].len,
        ) {
            (true, true) => (0, self.buffer[row].len),
            (true, false) => (0, self.size.0),
            (false, _) => (self.cursor.0 - self.size.0, self.cursor.0 - 1),
        };
        write!(
            self.stdout,
            "{}{}{}",
            termion::cursor::Goto(1, (row + 2) as u16),
            termion::clear::CurrentLine,
            &self.buffer[row].text[l..r],
        )
        .unwrap();
    }

    pub fn render(&mut self) {
        match self.mode {
            Mode::Insert => {
                write!(
                    self.stdout,
                    "{}{}INSERT MODE{}",
                    termion::cursor::Goto(1, 1),
                    termion::clear::CurrentLine,
                    termion::cursor::Goto(self.cursor.0 as u16, (self.cursor.1 + 1) as u16)
                )
                .unwrap();
                if !self.full {
                    assert!(self.cursor.1 - 1 < self.buffer.len());
                    self.render_line(self.cursor.1 - 1);
                    write!(
                        self.stdout,
                        "{}",
                        termion::cursor::Goto(self.cursor.0 as u16, (self.cursor.1 + 1) as u16)
                    )
                    .unwrap();
                } else {
                    for i in 0..self.buffer.len() {
                        self.render_line(i);
                    }
                    write!(
                        self.stdout,
                        "{}{}{}",
                        termion::cursor::Goto(1, (self.buffer.len() + 2) as u16),
                        termion::clear::CurrentLine,
                        termion::cursor::Goto(self.cursor.0 as u16, (self.cursor.1 + 1) as u16)
                    )
                    .unwrap();
                    self.full = false;
                }
            }
            Mode::Base => write!(
                self.stdout,
                "{}{}BASE MODE",
                termion::cursor::Goto(1, 1),
                termion::clear::CurrentLine
            )
            .unwrap(),
            Mode::Command => {
                write!(
                    self.stdout,
                    "{}{}COMMAND MODE:",
                    termion::cursor::Goto(1, 1),
                    termion::clear::CurrentLine
                )
                .unwrap();
                for c in &self.cmd {
                    write!(self.stdout, "{}", c).unwrap();
                }
            }
        }
        self.stdout.flush().unwrap()
    }

    fn update_cursor(&mut self, dir: CurMov) {
        match dir {
            CurMov::Up => {
                if self.cursor.1 > 1 {
                    self.cursor.1 -= 1;
                    if self.cursor.0 > self.buffer[self.cursor.1 - 1].len + 1 {
                        self.cursor.0 = self.buffer[self.cursor.1 - 1].len + 1
                    }
                }
            }
            CurMov::Down => {
                if self.cursor.1 < self.buffer.len() {
                    self.cursor.1 += 1;
                    if self.cursor.0 > self.buffer[self.cursor.1 - 1].len + 1 {
                        self.cursor.0 = self.buffer[self.cursor.1 - 1].len + 1
                    }
                }
            }
            CurMov::Left => {
                if self.cursor.0 > 1 {
                    self.cursor.0 -= 1
                }
            }
            CurMov::Right => {
                if self.cursor.0 <= self.buffer[self.cursor.1 - 1].len {
                    self.cursor.0 += 1
                }
            }
            CurMov::Goto(x, y) => self.cursor = (x, y),
        }
    }

    pub fn key_handle(&mut self, key: &Key) {
        match self.mode {
            Mode::Base => self.base_mode(key),
            Mode::Insert => self.insert_mode(key),
            Mode::Command => self.command_mode(key),
        }
    }

    fn base_mode(&mut self, key: &Key) {
        match key {
            Key::Char('i') => self.mode = Mode::Insert,
            Key::Char(':') => self.mode = Mode::Command,
            _ => (),
        }
    }

    fn insert_mode(&mut self, key: &Key) {
        match key {
            Key::Esc => self.mode = Mode::Base,
            Key::Up => self.update_cursor(CurMov::Up),
            Key::Down => self.update_cursor(CurMov::Down),
            Key::Left => self.update_cursor(CurMov::Left),
            Key::Right => self.update_cursor(CurMov::Right),
            Key::Delete | Key::Backspace => {
                if self.cursor.0 > 1 {
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
                    self.buffer.insert(self.cursor.1, Row::new());
                    self.update_cursor(CurMov::Goto(1, self.cursor.1 + 1));
                } else {
                    let row = self.buffer[self.cursor.1 - 1].clone();
                    self.buffer[self.cursor.1 - 1]
                        .text
                        .truncate(self.cursor.0 - 1);
                    self.buffer[self.cursor.1 - 1].len = self.cursor.0 - 1;

                    let new_row = Row {
                        text: (&row.text[self.cursor.0 - 1..row.len]).to_string(),
                        len: row.len + 1 - self.cursor.0,
                    };
                    self.buffer.insert(self.cursor.1, new_row);
                    self.update_cursor(CurMov::Goto(1, self.cursor.1 + 1));
                    self.full = true;
                }
            }
            Key::Char(c) => {
                self.buffer[self.cursor.1 - 1]
                    .text
                    .insert(self.cursor.0 - 1, *c);
                self.buffer[self.cursor.1 - 1].len += 1;
                self.update_cursor(CurMov::Right);
            }
            _ => (),
        }
    }

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
                            } else if let Some(file) = &self.name {
                                write_file(file.as_str(), &self.buffer).unwrap();
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

fn read_file(name: &str, out: &mut Vec<Row>) -> std::io::Result<()> {
    let file = std::fs::OpenOptions::new().read(true).open(name)?;

    let reader = BufReader::new(file);

    for line in reader.lines() {
        let text = line?;
        let len = text.len();
        out.push(Row {
            text: text,
            len: len,
        })
    }
    Ok(())
}

fn write_file(name: &str, text: &Vec<Row>) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(name)?;
    for r in text {
        file.write(r.text.as_bytes())?;
        file.write(b"\n")?;
    }
    file.flush()
}
