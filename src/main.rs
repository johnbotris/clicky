#![feature(never_type)]
#![feature(str_split_once)]

mod app;
mod gui;
mod midi;
mod opts;
mod tui;

use app::App;

use anyhow::Result;

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
    let connection = midi::connect(
        if let Some(name) = &opts.port_name {
            midi::ConnectBy::Name(name.clone())
        } else if let Some(index) = opts.port_index {
            midi::ConnectBy::Index(index)
        } else {
            midi::ConnectBy::Default
        },
        env!("CARGO_PKG_NAME"),
    )?;

    let app = gui::GuiApp::new(opts);

    app.run(connection)
}
