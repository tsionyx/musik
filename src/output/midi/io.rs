use std::io::{ErrorKind, Write};

use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

fn get_default_port(out: &MidiOutput) -> Option<MidiOutputPort> {
    let ports = out.ports();
    if ports.is_empty() {
        return None;
    }
    for p in &ports {
        // TODO: log.info
        println!("{:?}", out.port_name(p));
    }

    if ports.len() == 1 {
        ports.into_iter().next()
    } else {
        let mut without_midi_through = ports.iter().filter(|p| {
            out.port_name(p)
                .map_or(false, |name| !name.contains("Midi Through"))
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

        // TODO: log.info
        println!("Choosing {:?}", out.port_name(&port));
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
