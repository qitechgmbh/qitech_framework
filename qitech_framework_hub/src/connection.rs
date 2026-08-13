use std::time::Instant;

pub struct RuntimeConnection {
    state: State,
}

impl RuntimeConnection {
    pub(crate) fn new() -> Self {
        Self {
            state: State::Disconnected {
                since: std::time::Instant::now(),
            },
        }
    }

    // internal-only mutation
    pub(crate) fn mark_connected(&mut self, session: HubSession) {
        self.state = State::Connected(session);
    }

    pub(crate) fn mark_disconnected(&mut self) {
        self.state = State::Disconnected {
            since: std::time::Instant::now(),
        };
    }

    // external code only gets a read-only, coarse view
    pub fn state(&self) -> State {
        match &self.state {
            State::Disconnected { .. } => ConnectionStatus::Disconnected,
            State::Handshaking(_) => ConnectionStatus::Handshaking,
            State::Connected(_) => ConnectionStatus::Connected,
        }
    }
}

enum State {
    Disconnected { since: Instant },
    Handshaking(HubHandshake),
    Connected(HubSession),
}

#[non_exhaustive]
pub enum ConnectionStatus {
    Disconnected,
    Handshaking,
    Connected,
}
