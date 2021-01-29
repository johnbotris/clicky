use std::collections::HashSet;
use winit::{
    event::{self, ElementState::*, Event, KeyboardInput, ScanCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut keys_pressed = HashSet::new();
    let mut sustained = HashSet::new(); // this should be handled by the receiver using midi cc sustain
    let mut sustain_held = false;

    let send_note_on = |note| println!("note on {}", note);
    let send_note_off = |note| println!("note off {}", note);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => {
                match process_keyboard_input(input, &mut keys_pressed) {
                    Exit => {
                        *control_flow = ControlFlow::Exit
                    },
                    NoteOn(note) => send_note_on(note),
                    NoteOff(note) => {
                        if sustain_held {
                            sustained.insert(note);
                        }
                        else {
                            send_note_off(note);
                        }
                    },
                    Chord => {},
                    KillAll => {},
                    SustainOn => sustain_held = true,
                    SustainOff => {
                        sustain_held = false;
                        for note in sustained.drain() {
                            send_note_off(note);
                        }
                    },
                    Nothing => {}
                }
            }
            _ => {}
        },
        _ => {}
    });
}

fn process_keyboard_input(
    input: KeyboardInput,
    keys_pressed: &mut HashSet<ScanCode>,
) -> ProcessedKeyboardInput {
    use event::VirtualKeyCode::*;

    if let Some(Escape) = input.virtual_keycode {
        return Exit;
    }

    match input.state {
        Pressed => {
            if input.virtual_keycode == Some(Delete) {
                KillAll
            } else if keys_pressed.insert(input.scancode) {
                if input.virtual_keycode == Some(Space) {
                    SustainOn
                } else {
                    NoteOn(input.scancode)
                }
            } else {
                Nothing
            }
        }
        Released => {
            if keys_pressed.remove(&input.scancode) {
                if input.virtual_keycode == Some(Space) {
                    SustainOff
                }
                else {
                    NoteOff(input.scancode)
                }
            } else {
                Nothing
            }
        }
    }
}

enum ProcessedKeyboardInput {
    Exit,
    NoteOn(ScanCode),
    NoteOff(ScanCode),
    Chord,
    KillAll,
    SustainOn,
    SustainOff,
    Nothing,
}
use ProcessedKeyboardInput::*;
