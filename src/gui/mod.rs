use std::{collections::HashSet, convert::TryInto};
use winit::{
    event::{self, ElementState::*, Event, KeyboardInput, ScanCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wmidi::Note;

use crate::{app::Controller, opts};

pub struct GuiApp {
    opts: opts::Opts,
    keys_pressed: HashSet<ScanCode>,
}

impl GuiApp {
    pub fn new(opts: opts::Opts) -> Self {
        Self {
            opts,
            keys_pressed: HashSet::new(),
        }
    }
    /// Convert a keyboard input into some kind of action for the MIDI controller, or other
    fn process_keyboard_input(&mut self, input: KeyboardInput) -> Option<ProcessedKeyboardInput> {
        use event::VirtualKeyCode::*;

        log::trace!("keyboard input {:?}", input);

        if let Some(Escape) = input.virtual_keycode {
            return Some(Exit);
        }

        match input.state {
            Pressed => {
                if input.virtual_keycode == Some(Delete) {
                    Some(KillAll)
                } else if self.keys_pressed.insert(input.scancode) {
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
                if self.keys_pressed.remove(&input.scancode) {
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
}

impl crate::app::App for GuiApp {
    fn run(mut self, mut controller: Box<dyn Controller>) -> anyhow::Result<()> {
        let event_loop = EventLoop::new();
        let _window = WindowBuilder::new()
            .with_title(env!("CARGO_PKG_NAME"))
            .build(&event_loop)
            .unwrap();

        let exit = |control_flow: &mut ControlFlow| {
            log::info!("Exiting...");
            *control_flow = ControlFlow::Exit
        };

        let channel = self.opts.channel;
        let velocity = wmidi::U7::MAX;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => exit(control_flow),
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(action) = self.process_keyboard_input(input) {
                            let result = match action {
                                Exit => Ok(exit(control_flow)),
                                NoteOn(note) => controller.note_on(note, velocity),
                                NoteOff(note) => controller.note_off(note),
                                Chord => Ok(()),
                                KillAll => Ok(()),
                                SustainOn => controller.sustain_on(),
                                SustainOff => controller.sustain_off(),
                            };

                            if let Err(e) = result {
                                log::error!("Fatal error: {}", e);
                                exit(control_flow);
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        });
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
