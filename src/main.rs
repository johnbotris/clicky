use std::collections::HashSet;
use winit::{
    event::{self, ElementState::*, Event, KeyboardInput, WindowEvent, ScanCode},
    event_loop::{EventLoop, ControlFlow},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut keyspressed = HashSet::new();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event,
            ..
        } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => match process_keyboard_input(input, &mut keyspressed) {
                Exit => *control_flow = ControlFlow::Exit,
                NoteOn(note) => println!("todo: note on {}", note),
                NoteOff(note) => println!("todo: note off {}", note),
                Nothing => {}
            },
            _ => {}
        }
        _ => {}
    });
}

fn process_keyboard_input(input: KeyboardInput, keyspressed: &mut HashSet<ScanCode>) -> ProcessedKeyboardInput {
    use event::VirtualKeyCode::*;

    if let Some(Escape) = input.virtual_keycode {
        return Exit
    }

    match input.state {
            Pressed => if keyspressed.insert(input.scancode) {
                NoteOn(input.scancode)
            } else { Nothing },
            Released => if keyspressed.remove(&input.scancode) { // I don't think it's possible for this to be false, whatevs
                NoteOff(input.scancode)
            } else { Nothing }
    }
}

enum ProcessedKeyboardInput {
    Exit,
    NoteOn(ScanCode),
    NoteOff(ScanCode),
    Nothing
}
use ProcessedKeyboardInput::*;
