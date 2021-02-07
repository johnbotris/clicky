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
    pub verbose: u64,

    /// Output less information, can be passed multiple times
    #[structopt(short, parse(from_occurrences))]
    pub quiet: u64,
}

fn get_channel(s: &str) -> anyhow::Result<wmidi::Channel> {
    Ok(wmidi::Channel::from_index(s.parse::<u8>()?)?)
}
