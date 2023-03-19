use std::io::{BufRead, BufReader, Write};

#[derive(Debug, Clone)]
pub struct Row {
    pub text: String,
    pub len: usize,
}

impl Row {
    pub fn new() -> Row {
        Row {
            text: String::new(),
            len: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Insert,  // insert mode
    Base,    // base mode
    Command, // command input mode
}

#[derive(Debug, Clone, Copy)]
pub enum CurMov {
    Up,
    Down,
    Left,
    Right,
    Goto(usize, usize),
}

pub fn read_file(name: &str, out: &mut Vec<Row>) -> std::io::Result<()> {
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

pub fn write_file(name: &str, text: &Vec<Row>) -> std::io::Result<()> {
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
