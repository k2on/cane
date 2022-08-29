use std::collections::HashMap;
use std::io::{stdin, stdout, Error, ErrorKind};
use std::io::Write;
use std::env;
use std::fs;
use std::io;
use std::fmt;
use std::process::{Command, Output};
use std::str::from_utf8;

use termion::{clear, cursor};
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use termion::event::Key;


fn main() {
    start_new_cool_repl();

    // let command =  "cat myfile";
    // let result = get_command_result(command);
    // match result {
    //     Ok(res) => println!("{}", res),
    //     Err(err) => eprintln!("{}", err),
    // }

}

fn get_command_result(command: &str) -> Result<String, Error> {

    let result = Command::new("sh")
        .args(["-c", command])
        .output();
    
    match result {
        Result::Err(err) => Result::Err(err),
        Result::Ok(output) => handle_output(output),
    }
}

fn handle_output(output: Output) -> Result<String, Error> {
    if let Some(code) = output.status.code() {
        let stdout = from_utf8(output.stdout.as_slice()).unwrap().to_string();
        let stderr = from_utf8(output.stderr.as_slice()).unwrap().to_string();
        if code != 0 {
            Result::Err(Error::new(ErrorKind::Other, stderr))
        } else if stderr != "" {
            Result::Err(Error::new(ErrorKind::Other, stderr))
        } else {
            Result::Ok(stdout)
        }
    } else {
        Result::Err(Error::new(ErrorKind::InvalidData, "No command status code"))
    }
}


fn start_new_cool_repl() {
    // TODO: check if the stdin is tty
    // If it is not maybe switch to the old/simplified REPL
    let prompt = "> ";
    let mut stdout = stdout().into_raw_mode().unwrap();
    let stdin = stdin();
    write!(stdout, "{}", prompt).unwrap();
    stdout.flush().unwrap();

    let mut new_cool_repl: NewCoolRepl = Default::default();

    for key in stdin.keys() {
        match key.unwrap() {
            Key::Char('\n') => {
                write!(stdout, "\r\n").unwrap();
                if &new_cool_repl.take() == "quit" {
                    break
                }
            }
            Key::Ctrl('a') | Key::Home => new_cool_repl.home(),
            Key::Ctrl('e') | Key::End => new_cool_repl.end(),
            Key::Left => new_cool_repl.left_char(),
            Key::Right => new_cool_repl.right_char(),
            Key::Ctrl('j') | Key::Down => new_cool_repl.down(),
            Key::Ctrl('k') | Key::Up => new_cool_repl.up(),
            Key::Ctrl('c') => {
                write!(stdout, "^C\r\n").unwrap();
                break;
            }
            Key::Ctrl('b') => new_cool_repl.left_word(),
            Key::Ctrl('w') => new_cool_repl.right_word(),
            Key::Char(key) => {
                new_cool_repl.insert_char(key);
                new_cool_repl.popup.clear();



                let command =  "cat myfile";
                let result = get_command_result(command);
                match result {
                    Ok(res) => {
                        for line in res.lines() {
                            new_cool_repl.popup.push(format!("{}", line));
                        }
                    },
                    Err(err) => {
                        eprintln!("{}", err); 
                    },
                }
            },
            Key::Backspace => new_cool_repl.backspace(),
            _ => {},
        }
        new_cool_repl.render(prompt, &mut stdout).unwrap();
        stdout.flush().unwrap();
    }
}



#[derive(Default)]
pub struct NewCoolRepl {
    pub buffer: Vec<char>,
    pub buffer_cursor: usize,
    pub popup: Vec<String>,
    pub popup_cursor: usize,
}

impl NewCoolRepl {
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.buffer_cursor = 0;
        self.popup.clear();
        self.popup_cursor = 0;
    }

    pub fn take(&mut self) -> String {
        let result = self.buffer.iter().collect();
        self.clear();
        result
    }

    pub fn insert_char(&mut self, x: char) {
        self.buffer.insert(self.buffer_cursor, x);
        self.buffer_cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.buffer_cursor > 0 {
            self.buffer.remove(self.buffer_cursor - 1);
            self.buffer_cursor -= 1;
        }
    }

    pub fn home(&mut self) {
        self.buffer_cursor = 0;
    }

    pub fn end(&mut self) {
        self.buffer_cursor = self.buffer.len();
    }

    pub fn up(&mut self) {
        if self.popup_cursor > 0 {
            self.popup_cursor -= 1
        }
    }

    pub fn down(&mut self) {
        if self.popup_cursor < self.popup.len() - 1 {
            self.popup_cursor += 1;
            self.buffer = self.popup[self.popup_cursor].chars().collect()
        }
    }

    pub fn left_word(&mut self) {
        while self.buffer_cursor > 0 && self.buffer_cursor <= self.buffer.len() && !self.buffer.get(self.buffer_cursor - 1).unwrap().is_alphanumeric() {
            self.buffer_cursor -= 1;
        }
        while self.buffer_cursor > 0 && self.buffer_cursor <= self.buffer.len() && self.buffer.get(self.buffer_cursor - 1).unwrap().is_alphanumeric() {
            self.buffer_cursor -= 1;
        }
    }

    pub fn right_word(&mut self) {
        while self.buffer_cursor < self.buffer.len() && !self.buffer.get(self.buffer_cursor).unwrap().is_alphanumeric() {
            self.buffer_cursor += 1;
        }
        while self.buffer_cursor < self.buffer.len() && self.buffer.get(self.buffer_cursor).unwrap().is_alphanumeric() {
            self.buffer_cursor += 1;
        }
    }

    pub fn left_char(&mut self) {
        if self.buffer_cursor > 0 {
            self.buffer_cursor -= 1;
        }
    }

    pub fn right_char(&mut self) {
        if self.buffer_cursor < self.buffer.len() {
            self.buffer_cursor += 1;
        }
    }

    pub fn render(&self, prompt: &str, sink: &mut impl Write) -> io::Result<()> {
        const POPUP_SIZE: usize = 5;
        let buffer: String = self.buffer.iter().collect();
        write!(sink, "\r{}{}{}\r\n", clear::AfterCursor, prompt, &buffer)?;
        for (index, line) in self.popup.iter().take(POPUP_SIZE).enumerate() {
            if index == self.popup_cursor {
                write!(sink, ">")?
            } else {
                write!(sink, " ")?
            }
            write!(sink, " {}\r\n", line)?;
        }
        write!(sink, "{}{}",
               cursor::Up((POPUP_SIZE.min(self.popup.len()) + 1).try_into().unwrap()),
               cursor::Right((prompt.len() + self.buffer_cursor).try_into().unwrap()))?;
        Ok(())
    }
}
