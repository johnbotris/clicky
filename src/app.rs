pub trait App {
    fn run(&mut self, handler: Box<dyn MessageHandler>) -> anyhow::Result<!>;
}

pub trait MessageHandler {
    fn handle(&mut self, message: Message);
}

// TODO Don't use MidiMessage directly here, later we might want to use OSC, or connect to some sockets or something cool like that
pub type Message<'a> = wmidi::MidiMessage<'a>;
