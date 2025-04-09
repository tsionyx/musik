use std::{
    fmt,
    io::{ErrorKind, Write},
};

use log::info;
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

fn get_default_port(out: &MidiOutput) -> Option<MidiOutputPort> {
    let ports = out.ports();
    if ports.is_empty() {
        info!("Not found any MIDI ports");
        return None;
    }

    info!(
        "Available ports: {:?}",
        ports.iter().map(|p| out.port_name(p)).collect::<Vec<_>>()
    );

    if ports.len() == 1 {
        ports.into_iter().next()
    } else {
        let mut without_midi_through = ports.iter().filter(|p| {
            out.port_name(p)
                .is_ok_and(|name| !name.contains("Midi Through"))
        });

        without_midi_through
            .next()
            .cloned()
            .or_else(|| ports.into_iter().next())
    }
}

type AnyError = Box<dyn std::error::Error>;

pub struct Connection {
    buf: Vec<u8>,
    inner: MidiOutputConnection,
}

impl Connection {
    pub fn get_default() -> Result<Self, AnyError> {
        let out = MidiOutput::new("musik library MIDI player")?;
        let port = get_default_port(&out).ok_or("Not found any MIDI output device")?;

        info!("Choosing {:?} for playing", out.port_name(&port));
        let conn = out.connect(&port, "playing Music")?;
        Ok(Self {
            inner: conn,
            buf: Vec::new(),
        })
    }
}

impl Write for Connection {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let size = buf.len();
        self.buf.extend_from_slice(buf);
        Ok(size)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner
            .send(&self.buf)
            .map_err(|err| std::io::Error::new(ErrorKind::InvalidData, err))?;
        self.buf.clear();
        Ok(())
    }
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .field("buf", &self.buf)
            .finish_non_exhaustive()
    }
}
