#![feature(box_syntax)]
#![feature(never_type)]
#![feature(str_split_once)]

mod app;
mod gui;
mod logging;
mod midi;
mod opts;
mod tui;

use app::{App, MessageHandler};

use anyhow::{anyhow, Result};

fn main() {
    let opts = opts::get_opts();

    if opts.list_midi_outputs {
        return midi::list_outputs();
    }

    match run(opts) {
        Ok(_) => (),
        Err(err) => log::error!("Error: {}", err),
    };
}

fn run(opts: opts::Opts) -> Result<!> {
    logging::init_logging(opts.quiet, opts.verbose);

    let handler: Box<dyn MessageHandler> = match &opts.mode {
        opts::Mode::midi => {
            let connect_by = if let Some(name) = &opts.port_name {
                midi::ConnectBy::Name(name.clone())
            } else if let Some(index) = opts.port_index {
                midi::ConnectBy::Index(index)
            } else {
                midi::ConnectBy::Default
            };
            box crate::midi::MidiHandler::new(opts.channel, connect_by, env!("CARGO_PKG_NAME"))?
        }
        _ => {
            return Err(anyhow!(
            "Mode {} isn't implemented yet. Feel free to send an email to complaints@johnbotr.is",
            opts.mode
        ))
        }
    };

    let mut app: Box<dyn App> = match opts.ui_mode {
        opts::UiMode::tui => box tui::TuiApp::new(opts),
        opts::UiMode::gui => box gui::GuiApp::new(opts),
    };

    app.run(handler)
}
