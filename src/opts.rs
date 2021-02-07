use clap::arg_enum;
use structopt::StructOpt;

pub fn get_opts() -> Opts {
    Opts::from_args()
}

#[derive(Debug, StructOpt)]
#[structopt(author, about)]
pub struct Opts {
    /// MIDI channel to run on
    #[structopt(short, long, parse(try_from_str = get_channel), default_value = "1")]
    pub channel: wmidi::Channel,

    /// Name of the port to connect to
    #[structopt(short, long)]
    pub port_name: Option<String>,

    /// Index of the port to connect to
    #[structopt(short = "i", long)]
    pub port_index: Option<usize>,

    /// List available MIDI output ports then exit
    #[structopt(short, long)]
    pub list_midi_outputs: bool,

    /// Output more information, can be passed multiple times
    #[structopt(short, parse(from_occurrences))]
    pub verbose: i8,

    /// Output less information, can be passed multiple times
    #[structopt(short, parse(from_occurrences))]
    pub quiet: i8,

    /// Output mode
    #[structopt(short, long, possible_values = &Mode::variants(), default_value = "midi", case_insensitive = true)]
    pub mode: Mode,

    /// Ui Mode (tui uses tui-rs, gui uses winit + ? (unimplemented))
    #[structopt(short, long, possible_values = &UiMode::variants(), default_value = "tui", case_insensitive = true)]
    pub ui_mode: UiMode,
}

arg_enum! {
    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    pub enum UiMode {
        tui,
        gui
    }
}

arg_enum! {
    #[allow(non_camel_case_types)]
    #[derive(Debug, Clone)]
    pub enum Mode {
        midi,
        osc,
        socket
    }
}

fn get_channel(s: &str) -> anyhow::Result<wmidi::Channel> {
    Ok(wmidi::Channel::from_index(s.parse::<u8>()?)?)
}
