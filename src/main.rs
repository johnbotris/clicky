#![feature(box_syntax)]
#![feature(never_type)]
#![feature(str_split_once)]

mod app;
mod gui;
mod logging;
mod midi;
mod opts;
mod tui;

use app::{App, Controller};

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

fn run(opts: opts::Opts) -> Result<()> {
    logging::init_logging(opts.quiet, opts.verbose);

    let controller: Box<dyn Controller> = match &opts.mode {
        opts::Mode::midi => get_midi_controller(&opts)?,
        _ => {
            return Err(anyhow!(
            "Mode {} isn't implemented yet. Feel free to send an email to complaints@johnbotr.is",
            opts.mode
        ))
        }
    };

    match opts.ui_mode {
        opts::UiMode::tui => run_tui(opts, controller),
        opts::UiMode::gui => run_gui(opts, controller),
    }
}

fn get_midi_controller(opts: &opts::Opts) -> Result<Box<dyn Controller>> {
    let connect_by = if let Some(name) = &opts.port_name {
        midi::ConnectBy::Name(name.clone())
    } else if let Some(index) = opts.port_index {
        midi::ConnectBy::Index(index)
    } else {
        midi::ConnectBy::Default
    };
    Ok(
        box crate::midi::MidiController::new(opts.channel, connect_by, env!("CARGO_PKG_NAME"))?
            as Box<dyn Controller>,
    )
}

fn run_tui(opts: opts::Opts, controller: Box<dyn Controller>) -> Result<()> {
    #[cfg(any(feature = "default", feature = "tui-mode"))]
    let result = tui::TuiApp::new(opts).run(controller);

    #[cfg(not(any(feature = "default", feature = "tui-mode")))]
    let result = Err(anyhow!(
        "Build {} with \"tui-mode\" enabled if you want to use the TUI",
        env!("CARGO_PKG_NAME")
    ));

    result
}

fn run_gui(opts: opts::Opts, controller: Box<dyn Controller>) -> Result<()> {
    #[cfg(feature = "gui-mode")]
    let result = gui::GuiApp::new(opts).run(controller);

    #[cfg(not(feature = "gui-mode"))]
    let result = Err(anyhow!(
        "Build {} with \"gui-mode\" enabled if you want to use the GUI",
        env!("CARGO_PKG_NAME")
    ));

    result
}
