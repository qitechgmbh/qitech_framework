use crate::MachineEntry;

pub enum Cursor {
    Tab,
    Config { field: usize },
    State { field: usize },
    Measurement { field: usize },
}

impl Cursor {
    pub fn is_config(&self) -> bool {
        matches!(self, Cursor::Config { .. })
    }

    pub fn is_state(&self) -> bool {
        matches!(self, Cursor::State { .. })
    }

    pub fn is_measurement(&self) -> bool {
        matches!(self, Cursor::Measurement { .. })
    }

    pub fn up(&mut self, machine: &MachineEntry) -> bool {
        match self {
            Cursor::Tab => return true,

            Cursor::Config { field } => {
                if *field > 0 {
                    *field -= 1;
                } else {
                    *self = Cursor::Tab;
                }
            }

            Cursor::State { field } => {
                if *field > 0 {
                    *field -= 1;
                } else if !machine.config.is_empty() {
                    *self = Cursor::Config {
                        field: machine.config.len() - 1,
                    };
                } else {
                    *self = Cursor::Tab;
                }
            }

            Cursor::Measurement { field } => {
                if *field > 0 {
                    *field -= 1;
                } else if !machine.state.is_empty() {
                    *self = Cursor::State {
                        field: machine.state.len() - 1,
                    };
                } else if !machine.config.is_empty() {
                    *self = Cursor::Config {
                        field: machine.config.len() - 1,
                    };
                } else {
                    *self = Cursor::Tab;
                }
            }
        }

        false
    }

    pub fn down(&mut self, machine: &MachineEntry) {
        match self {
            Cursor::Tab => {
                if !machine.config.is_empty() {
                    *self = Cursor::Config { field: 0 };
                } else if !machine.state.is_empty() {
                    *self = Cursor::State { field: 0 };
                } else if !machine.measurements.is_empty() {
                    *self = Cursor::Measurement { field: 0 };
                }
            }

            Cursor::Config { field } => {
                if *field + 1 < machine.config.len() {
                    *field += 1;
                } else if !machine.state.is_empty() {
                    *self = Cursor::State { field: 0 };
                } else if !machine.measurements.is_empty() {
                    *self = Cursor::Measurement { field: 0 };
                }
            }

            Cursor::State { field } => {
                if *field + 1 < machine.state.len() {
                    *field += 1;
                } else if !machine.measurements.is_empty() {
                    *self = Cursor::Measurement { field: 0 };
                }
            }

            Cursor::Measurement { field } => {
                if *field + 1 < machine.measurements.len() {
                    *field += 1;
                }
            }
        }
    }
}
