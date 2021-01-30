#![feature(never_type)]
#![feature(str_split_once)]

use core::convert::TryInto;
use std::collections::HashSet;
use anyhow::{anyhow, Result};
use winit::{
    event::{self, ElementState::*, Event, KeyboardInput, ScanCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use midir::{MidiOutput, MidiOutputPort};
use wmidi::{MidiMessage, Note};

const MIDI_OUTPUT_NAME: &str = env!("CARGO_PKG_NAME");

mod opts {
    use structopt::StructOpt;

    pub fn get_opts() -> Opts{
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
        pub list_midi_outputs: bool
    }

    fn get_channel(s: &str) -> anyhow::Result<wmidi::Channel> {
        Ok(wmidi::Channel::from_index(s.parse::<u8>()?)?)
    }
}

fn main() {
    use simple_logger::SimpleLogger;
    use log::LevelFilter;

    let opts = opts::get_opts();

    if opts.list_midi_outputs {
        list_midi_outputs();
        return
    }


    match SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init() {
            Ok(_) => log::trace!("logging initialized"),
            Err(e) => eprintln!("Failed to initialize logging: {}", e)
        }

    match run(opts) {
        Ok(_) => (),
        Err(err) => log::error!("Error: {}", err)
    };
}

fn run(opts: opts::Opts) -> Result<!> {


    let output = MidiOutput::new(MIDI_OUTPUT_NAME)?;
    let ports: &[MidiOutputPort] = &output.ports();
    let port = if let Some(name) = opts.port_name {
        unimplemented!()
    } else if let Some(index) = opts.port_index {
        unimplemented!()
    } else {
        match ports {
            [] => Err(anyhow!("No available MIDI outputs")),
            [port, ..] => Ok(port),
        }?
    };


    let port_name = output.port_name(&port)
        .unwrap_or(String::from("<unknown>"));

    let mut connection = output.connect(&port, MIDI_OUTPUT_NAME)
        .map_err(|e| anyhow!("Couldn't connect to MIDI output port \"{}\": {}", port_name, e))?;

    println!("Connected to midi port \"{}\"", port_name);

    let event_loop = EventLoop::new();
    let _window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut keys_pressed = HashSet::new();
    let mut sustained = HashSet::new(); // this should be handled by the receiver using midi cc sustain
    let mut sustain_held = false;

    let mut send_midi = move |message: MidiMessage | {
        log::debug!("midi message: {:?}", message);
        let mut bytes = [0u8, 0, 0];
        match message.copy_to_slice(&mut bytes) {
            Ok(length) => {
                log::trace!("bytes: {:?}, length: {}", bytes, length);
                if let Err(e) = connection.send(&bytes) {
                    log::error!("Error sending MIDI message {:?}: {}", message, e);
                };
            },
            Err(err) => log::error!("Error generating MIDI bytes: {}", err)
        };
    };

    let exit = |control_flow: &mut ControlFlow| {
        log::info!("Exiting...");
        *control_flow = ControlFlow::Exit
    };

    let channel = opts.channel;
    let velocity = 127u8.try_into().unwrap();


    event_loop.run(move |event, _, control_flow| match event {
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
                            }
                            else {
                                send_midi(MidiMessage::NoteOff(channel, note, velocity));
                            }
                        },
                        Chord => {},
                        KillAll => {},
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
    });
}

fn get_midi_note(scancode: ScanCode) -> Option<Note> {
    Some(Note::C1)
}

fn process_keyboard_input(
    input: KeyboardInput,
    keys_pressed: &mut HashSet<ScanCode>,
    ) -> Option<ProcessedKeyboardInput> {
    use event::VirtualKeyCode::*;

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
                    get_midi_note(input.scancode).map(NoteOn)
                }
            } else {
                None
            }
        }
        Released => {
            if keys_pressed.remove(&input.scancode) {
                if input.virtual_keycode == Some(Space) {
                    Some(SustainOff)
                }
                else {
                    get_midi_note(input.scancode).map(NoteOff)
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

fn list_midi_outputs() -> Result<()> {

    let output = MidiOutput::new(MIDI_OUTPUT_NAME)?;
    let ports = output.ports();

    println!("# -> name");
    println!("____________");
    for (i, port) in ports.iter().enumerate() {
        let name = output.port_name(port)?;
        if let Some((device, name)) = name.split_once(':') {
            println!("{} -> {}", i, device);
        };
    }

    Ok(())
}
