#![cfg(any(feature = "default", feature = "tui-mode"))]
use std::io::{self, Read, Write};
use termion::{input::TermRead, raw::IntoRawMode};
use tui::{backend::TermionBackend, Terminal};

use crate::{
    app::{App, MessageHandler},
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
    fn run(&mut self, handler: Box<dyn MessageHandler>) -> Result<!> {
        Err(anyhow!("whhoooop"))
    }
}

fn example() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode().unwrap();

    for event in stdin.events() {
        match event {
            Ok(event) => {
                write!(stdout, "{:?}\r\n", event);
            }
            Err(e) => {}
        }
    }
}
