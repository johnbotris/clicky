use crate::app::MessageHandler;
use anyhow::{anyhow, Result};
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};
use wmidi::{Channel, MidiMessage};

pub struct MidiHandler {
    channel: Channel,
    connection: MidiOutputConnection,
}

impl MidiHandler {
    pub fn new(channel: Channel, connect_by: ConnectBy, output_name: &str) -> Result<Self> {
        Ok(Self {
            channel,
            connection: connect(connect_by, output_name)?,
        })
    }

    fn send_midi(&mut self, message: MidiMessage) {
        log::trace!("midi message: {:?}", message);
        let mut bytes = [0u8, 0, 0];
        match message.copy_to_slice(&mut bytes) {
            Ok(length) => {
                if let Err(e) = self.connection.send(&bytes[..length]) {
                    log::warn!("Error sending MIDI message {:?}: {}", message, e);
                }
            }
            Err(err) => log::warn!("Error generating MIDI bytes: {}", err),
        };
    }
}

impl MessageHandler for MidiHandler {
    fn handle(&mut self, message: MidiMessage) {
        self.send_midi(message)
    }
}

pub enum ConnectBy {
    Name(String),
    Index(usize),
    Default,
}

pub fn connect(connect_by: ConnectBy, output_name: &str) -> Result<MidiOutputConnection> {
    let output = MidiOutput::new(output_name)?;
    let ports: &[MidiOutputPort] = &output.ports();
    let port = match connect_by {
        ConnectBy::Name(name) => {
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
        }
        ConnectBy::Index(index) => {
            log::debug!("Connecting to port with index {}", index);
            ports
                .get(index)
                .ok_or(anyhow!("Port index {} out of range", index))?
        }
        ConnectBy::Default => {
            log::debug!("Connecting to first available port");
            match ports {
                [] => Err(anyhow!("No available MIDI outputs")),
                [port, ..] => Ok(port),
            }?
        }
    };

    let port_name = output.port_name(&port).unwrap_or(String::from("<unknown>"));

    log::info!("Connecting to midi port \"{}\"", port_name);

    Ok(output.connect(&port, output_name).map_err(|e| {
        anyhow!(
            "Couldn't connect to MIDI output port \"{}\": {}",
            port_name,
            e
        )
    })?)
}

pub fn list_outputs() {
    let output = MidiOutput::new("").unwrap();
    let ports = output.ports();

    println!("Available MIDI outputs");
    for (i, port) in ports.iter().enumerate() {
        let name = output.port_name(port).unwrap_or(String::from("<unknown>"));
        if let Some((device, _name)) = name.split_once(':') {
            println!("{}: {}", i, device);
        };
    }
}
