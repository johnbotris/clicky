#![feature(never_type)]
#![feature(str_split_once)]

use anyhow::{anyhow, Result};
use core::convert::TryInto;
use midir::{MidiOutput, MidiOutputPort};
use std::{
    collections::HashSet,
    time::{Duration, Instant},
};
use winit::{
    event::{self, ElementState::*, Event, KeyboardInput, ScanCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wmidi::{MidiMessage, Note};

const MIDI_OUTPUT_NAME: &str = env!("CARGO_PKG_NAME");

mod opts {
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
}

fn main() {
    let opts = opts::get_opts();

    if opts.list_midi_outputs {
        let output = MidiOutput::new(MIDI_OUTPUT_NAME).unwrap();
        let ports = output.ports();

        println!("Available MIDI outputs");
        for (i, port) in ports.iter().enumerate() {
            let name = output.port_name(port).unwrap_or(String::from("<unknown>"));
            if let Some((device, _name)) = name.split_once(':') {
                println!("{}: {}", i, device);
            };
        }
        return;
    }

    init_logging(&opts);

    match run(opts) {
        Ok(_) => (),
        Err(err) => log::error!("Error: {}", err),
    };
}

fn init_logging(opts: &opts::Opts) {
    use log::LevelFilter::*;
    use std::cmp::{max, min};

    let default = 3;
    let verbose = opts.verbose as i64;
    let quiet = opts.quiet as i64;
    let level = min(max(default + verbose - quiet, 0), 5);
    assert!(level <= 5);
    match simple_logger::SimpleLogger::new()
        .with_level([Off, Error, Warn, Info, Debug, Trace][level as usize])
        .init()
    {
        Ok(_) => log::trace!("Logging initialized"),
        Err(e) => eprintln!("Failed to initialize logging: {}", e),
    }
}

fn run(opts: opts::Opts) -> Result<!> {
    let output = MidiOutput::new(MIDI_OUTPUT_NAME)?;
    let ports: &[MidiOutputPort] = &output.ports();
    let port = if let Some(name) = opts.port_name {
        log::debug!("Connecting to port with name {}", name);
        ports
            .iter()
            .find(|port| {
                output
                    .port_name(port)
                    .map(|port_name| port_name.starts_with(&name))
                    .unwrap_or(false)
            })
            .ok_or(anyhow!("No MIDI port named {}", name))?
    } else if let Some(index) = opts.port_index {
        log::debug!("Connecting to port with index {}", index);
        ports
            .get(index)
            .ok_or(anyhow!("Port index {} out of range", index))?
    } else {
        log::debug!("Connecting to first available port");
        match ports {
            [] => Err(anyhow!("No available MIDI outputs")),
            [port, ..] => Ok(port),
        }?
    };

    let port_name = output.port_name(&port).unwrap_or(String::from("<unknown>"));

    let mut connection = output.connect(&port, MIDI_OUTPUT_NAME).map_err(|e| {
        anyhow!(
            "Couldn't connect to MIDI output port \"{}\": {}",
            port_name,
            e
        )
    })?;

    log::info!("Connected to midi port \"{}\"", port_name);

    let event_loop = EventLoop::new();
    let _window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut keys_pressed = HashSet::new();
    let mut sustained = HashSet::new(); // TODO this should be handled by the receiver using midi cc sustain
    let mut sustain_held = false;

    let mut send_midi = move |message: MidiMessage| {
        log::trace!("midi message: {:?}", message);
        let mut bytes = [0u8, 0, 0];
        match message.copy_to_slice(&mut bytes) {
            Ok(length) => {
                if let Err(e) = connection.send(&bytes[..length]) {
                    log::warn!("Error sending MIDI message {:?}: {}", message, e);
                }
            }
            Err(err) => log::warn!("Error generating MIDI bytes: {}", err),
        };
    };

    let exit = |control_flow: &mut ControlFlow| {
        log::info!("Exiting...");
        *control_flow = ControlFlow::Exit
    };

    let channel = opts.channel;
    let velocity = 127u8.try_into().unwrap();

    event_loop.run(move |event, _, control_flow| {
        // *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(500));
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => exit(control_flow),
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(action) = process_keyboard_input(input, &mut keys_pressed) {
                        match action {
                            Exit => exit(control_flow),
                            NoteOn(note) => send_midi(MidiMessage::NoteOn(channel, note, velocity)),
                            NoteOff(note) => {
                                if sustain_held {
                                    sustained.insert(note);
                                } else {
                                    send_midi(MidiMessage::NoteOff(channel, note, velocity));
                                }
                            }
                            Chord => {}
                            KillAll => {}
                            SustainOn => sustain_held = true,
                            SustainOff => {
                                sustain_held = false;
                                for note in sustained.drain() {
                                    send_midi(MidiMessage::NoteOff(channel, note, velocity));
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    });
}

/// Convert a keyboard input into some kind of action for the MIDI controller, or other
fn process_keyboard_input(
    input: KeyboardInput,
    keys_pressed: &mut HashSet<ScanCode>,
) -> Option<ProcessedKeyboardInput> {
    use event::VirtualKeyCode::*;

    log::trace!("keyboard input {:?}", input);

    if let Some(Escape) = input.virtual_keycode {
        return Some(Exit);
    }

    match input.state {
        Pressed => {
            if input.virtual_keycode == Some(Delete) {
                Some(KillAll)
            } else if keys_pressed.insert(input.scancode) {
                if input.virtual_keycode == Some(Space) {
                    Some(SustainOn)
                } else {
                    get_midi_note(input).map(NoteOn)
                }
            } else {
                None
            }
        }
        Released => {
            if keys_pressed.remove(&input.scancode) {
                if input.virtual_keycode == Some(Space) {
                    Some(SustainOff)
                } else {
                    get_midi_note(input).map(NoteOff)
                }
            } else {
                None
            }
        }
    }
}

use ProcessedKeyboardInput::*;
enum ProcessedKeyboardInput {
    Exit,
    NoteOn(Note),
    NoteOff(Note),
    Chord,
    KillAll,
    SustainOn,
    SustainOff,
}

fn get_midi_note(input: KeyboardInput) -> Option<Note> {
    use core::convert::TryFrom;
    scancode_to_note_index(input.scancode)
        .map(wmidi::Note::try_from)
        .and_then(Result::ok)
}

// TODO Different note mappings
/// Somewhat arbitrary mapping from scancode to an index in range [0, ??], going left to right, bottom to top
fn scancode_to_note_index(code: ScanCode) -> Option<u8> {
    /*
     * (scancode key) on UK Qwerty:
       2 1, 3 2, 4 3, 5 4, 6 5, 7 6, 8 7, 9 8, 10 9, 11 0, 12 _, 13 +
       16 q, 17 w, 18 e, 19 r, 20 t, 21 y, 22 u, 23 i, 24 o, 25 p, 26 [, 27 ]
       30 a, 31 s, 32 d, 33 f, 34 g, 35 h, 36 j, 37 k, 38 l, 39 ;, 40 @, 41 ~
       44 z, 45 x, 46 c, 47 v, 48 b, 49 n, 50 m, 51 <, 52 >, 53 /

    */

    let index = match code {
        44..=53 => Some(code as u8 - 44),
        30..=41 => Some(code as u8 - 20),
        43 => Some(21),
        16..=27 => Some(code as u8 + 6),
        2..=13 => Some(code as u8 + 32),
        _ => None,
    };
    log::trace!("Scancode: {}, index: {:?}", code, index);
    index.map(|i| i + 36)
}
