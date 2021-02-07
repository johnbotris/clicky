use anyhow::{anyhow, Result};
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

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
