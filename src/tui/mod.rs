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
