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

    let mut app = match opts.ui_mode {
        opts::UiMode::tui => get_tui(opts),
        opts::UiMode::gui => get_gui(opts),
    }?;

    app.run(handler)
}

fn get_tui(opts: opts::Opts) -> Result<Box<dyn App>> {
    #[cfg(any(feature = "default", feature = "tui-mode"))]
    let result = Ok(box tui::TuiApp::new(opts) as Box<dyn App>);

    #[cfg(not(any(feature = "default", feature = "tui-mode")))]
    let result = Err(anyhow!(
        "Build {} with \"tui-mode\" enabled if you want to use the TUI",
        env!("CARGO_PKG_NAME")
    ));

    result
}

fn get_gui(opts: opts::Opts) -> Result<Box<dyn App>> {
    #[cfg(feature = "gui-mode")]
    let result = Ok(box gui::GuiApp::new(opts) as Box<dyn App>);

    #[cfg(not(feature = "gui-mode"))]
    let result = Err(anyhow!(
        "Build {} with \"gui-mode\" enabled if you want to use the GUI",
        env!("CARGO_PKG_NAME")
    ));

    result
}
