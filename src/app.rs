pub trait App {
    fn run(self, midi_connection: midir::MidiOutputConnection) -> anyhow::Result<!>;
}
