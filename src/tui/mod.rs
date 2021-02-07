#![cfg(any(feature = "default", feature = "tui-mode"))]
use std::io::{stdin, stdout, Read, Write};
use termion::{event::Key, input::TermRead, raw::IntoRawMode};
use tui::{backend::TermionBackend, Terminal};

use crate::{
    app::{App, Controller},
    opts::Opts,
};
use anyhow::{anyhow, Result};

pub struct TuiApp {
    opts: Opts,
}

impl TuiApp {
    pub fn new(opts: Opts) -> Self {
        Self { opts }
    }
}

impl App for TuiApp {
    fn run(mut self, handler: Box<dyn Controller>) -> Result<()> {
        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode()?;

        write!(
            stdout,
            "{}{}q to exit. Type stuff, use alt, and so on.{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            termion::cursor::Hide
        )?;
        stdout.flush()?;

        for c in stdin.keys() {
            match c? {
                Key::Esc => break,
                Key::Char(c) => println!("{}", c),
                Key::Alt(c) => println!("^{}", c),
                Key::Ctrl(c) => println!("*{}", c),
                Key::Left => println!("←"),
                Key::Right => println!("→"),
                Key::Up => println!("↑"),
                Key::Down => println!("↓"),
                Key::Backspace => println!("×"),
                _ => {}
            }
            stdout.flush()?;
        }

        write!(stdout, "{}", termion::cursor::Show)?;

        log::info!("Exiting");

        Ok(())
    }
}
