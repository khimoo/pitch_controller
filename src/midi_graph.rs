use alsa::seq::{Addr, ClientIter, PortCap, PortIter, PortSubscribe, Seq};
use std::fmt;
use std::sync::{Arc, Mutex};
use std::ffi::CString;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MidiEndpointId {
    pub client: i32,
    pub port: i32,
}

#[derive(Clone, Debug)]
pub struct MidiEndpoint {
    pub id: MidiEndpointId,
    pub name: String,
    pub can_read: bool,
    pub can_write: bool,
}

#[derive(Debug)]
pub enum MidiGraphError {
    Unavailable,
    Alsa(alsa::Error),
}

impl fmt::Display for MidiGraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MidiGraphError::Unavailable => write!(f, "MIDI graph backend unavailable"),
            MidiGraphError::Alsa(e) => write!(f, "ALSA error: {}", e),
        }
    }
}

impl std::error::Error for MidiGraphError {}

impl From<alsa::Error> for MidiGraphError {
    fn from(value: alsa::Error) -> Self {
        MidiGraphError::Alsa(value)
    }
}

/// Thin ALSA sequencer wrapper to enumerate ports and connect/disconnect them.
/// This keeps the surface small so we can swap the backend (e.g., JACK) later without touching UI code.
pub struct MidiGraph {
    seq: Arc<Mutex<Option<Seq>>>,
}

impl MidiGraph {
    pub fn new() -> Result<Self, MidiGraphError> {
        let seq = Seq::open(None, None, false)?;
        let name = CString::new("pitch_controller").expect("CString::new failed");
        seq.set_client_name(&name)?;
        Ok(Self {
            seq: Arc::new(Mutex::new(Some(seq))),
        })
    }

    fn with_seq<F, T>(&self, f: F) -> Result<T, MidiGraphError>
    where
        F: FnOnce(&Seq) -> Result<T, MidiGraphError>,
    {
        let guard = self.seq.lock().expect("seq mutex poisoned");
        if let Some(seq) = guard.as_ref() {
            f(seq)
        } else {
            Err(MidiGraphError::Unavailable)
        }
    }

    pub fn list_endpoints(&self) -> Result<Vec<MidiEndpoint>, MidiGraphError> {
        self.with_seq(|seq| {
            let mut endpoints = Vec::new();
            for client in ClientIter::new(seq) {
                let client_name = client
                    .get_name()
                    .unwrap_or("unknown-client")
                    .to_string();
                for port in PortIter::new(seq, client.get_client()) {
                    // Skip if no capabilities (sometimes metadata-only)
                    let caps = port.get_capability();
                    let can_read = caps.contains(PortCap::READ) || caps.contains(PortCap::SUBS_READ);
                    let can_write = caps.contains(PortCap::WRITE) || caps.contains(PortCap::SUBS_WRITE);
                    if !(can_read || can_write) {
                        continue;
                    }

                    let port_name = port
                        .get_name()
                        .unwrap_or("unknown-port")
                        .to_string();
                    let name = if port_name.is_empty() {
                        client_name.clone()
                    } else {
                        format!("{}: {}", client_name, port_name)
                    };

                    endpoints.push(MidiEndpoint {
                        id: MidiEndpointId {
                            client: port.get_client() as i32,
                            port: port.get_port() as i32,
                        },
                        name,
                        can_read,
                        can_write,
                    });
                }
            }
            Ok(endpoints)
        })
    }

    pub fn connect(&self, src: &MidiEndpointId, dst: &MidiEndpointId) -> Result<(), MidiGraphError> {
        self.with_seq(|seq| {
            let subs = PortSubscribe::empty()?;
            subs.set_sender(Addr {
                client: src.client,
                port: src.port,
            });
            subs.set_dest(Addr {
                client: dst.client,
                port: dst.port,
            });
            seq.subscribe_port(&subs)?;
            Ok(())
        })
    }

    pub fn disconnect(
        &self,
        src: &MidiEndpointId,
        dst: &MidiEndpointId,
    ) -> Result<(), MidiGraphError> {
        self.with_seq(|seq| {
            let sender = Addr {
                client: src.client,
                port: src.port,
            };
            let dest = Addr {
                client: dst.client,
                port: dst.port,
            };
            seq.unsubscribe_port(sender, dest)?;
            Ok(())
        })
    }
}