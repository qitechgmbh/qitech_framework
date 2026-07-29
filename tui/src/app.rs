use std::collections::HashMap;

use crossterm::event::KeyCode;
use qitech_framework::MachineIdentification;
use qitech_framework::runtime::bridge::CrossbeamHandle;
use qitech_framework::runtime::bridge::CrossbeamRuntimeInitEvent;
use qitech_framework_common::MachineSchema;
use qitech_framework_common::RuntimeReport;
use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Widget;

use crate::pages::MachinesPage;
use crate::pages::Page;
use crate::widgets::AppWidget;
use crate::widgets::AppWidgetState;

pub enum RuntimeStatus {
    Offline,
    Starting,
    Running,
}

enum Mode {
    Navigate(u16),
    Edit { widget: u16, buffer: String },
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum VerticalPosition {
    Status,
    Tab,
    Page,
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum TabPosition {
    Machines,
    EtherCAT,
    Modbus,
    Logs,
}

pub struct App {
    pub runtime_status: RuntimeStatus,

    // --- positions ---
    pub mode: Mode,
    pub pos_t: TabPosition,
    pub pos_v: VerticalPosition,

    // --- pages ---
    pub page_machines: MachinesPage,

    // --- misc ---
    pub running: bool,

    // --- new ---
    pub widgets: Vec<Box<dyn AppWidget>>,
    pub widget_pos: usize,
}

impl App {
    pub const TABS: [&'static str; 4] = ["Machines", "EtherCAT", "Modbus", "Logs"];

    pub fn display2(&mut self, frame: &mut Frame) {
        // --- draw outer box ---
        const TITLE: &str = " QiTech Control (Terminal Edition) ";
        let outer = Block::default().borders(Borders::ALL).title(TITLE);
        frame.render_widget(&outer, frame.area());

        let inner = outer.inner(frame.area());

        let mut constraints = Vec::new();
        for (i, widget) in self.widgets.iter().enumerate() {
            constraints.push(widget.height());
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        // --- peng ---
        for (i, widget) in self.widgets.iter().enumerate() {
            let state = if i == self.widget_pos {
                AppWidgetState::InFocus
            } else {
                AppWidgetState::NoFocus
            };

            widget.display(shared, state, chunks[i], frame);
        }
    }

    pub fn new(schemas: HashMap<MachineIdentification, MachineSchema>) -> Self {
        Self {
            running: true,
            pos_t: TabPosition::Machines,
            pos_v: VerticalPosition::Status,
            runtime_status: RuntimeStatus::Offline,
            page_machines: MachinesPage::new(schemas),
        }
    }

    fn selected_page(&mut self) -> &mut dyn Page {
        match self.pos_t {
            TabPosition::Machines => &mut self.page_machines,
            _ => todo!(),
        }
    }

    pub fn running(&self) -> bool {
        self.running
    }

    pub fn handle_key(&mut self, key: KeyCode, handle: &mut CrossbeamHandle) {
        _ = handle;

        match key {
            // --- exit ---
            KeyCode::Char('q') => self.running = false,

            KeyCode::Up => {
                self.pos_v = match self.pos_v {
                    VerticalPosition::Status => VerticalPosition::Status,
                    VerticalPosition::Tab => VerticalPosition::Status,
                    VerticalPosition::Page => {
                        let exited = self.selected_page().up();

                        if exited {
                            VerticalPosition::Tab
                        } else {
                            VerticalPosition::Page
                        }
                    }
                }
            }
            KeyCode::Down => {
                self.pos_v = match self.pos_v {
                    VerticalPosition::Status => VerticalPosition::Tab,
                    VerticalPosition::Tab => {
                        self.selected_page().down();
                        VerticalPosition::Page
                    }
                    VerticalPosition::Page => {
                        self.selected_page().down();
                        VerticalPosition::Page
                    }
                }
            }
            KeyCode::Left => match self.pos_v {
                VerticalPosition::Tab => {
                    self.pos_t = match self.pos_t {
                        TabPosition::Machines => TabPosition::Machines,
                        TabPosition::EtherCAT => TabPosition::Machines,
                        TabPosition::Modbus => TabPosition::EtherCAT,
                        TabPosition::Logs => TabPosition::Modbus,
                    }
                }
                VerticalPosition::Page => {
                    self.selected_page().left();
                }
                _ => {}
            },
            KeyCode::Right => match self.pos_v {
                VerticalPosition::Tab => {
                    self.pos_t = match self.pos_t {
                        TabPosition::Machines => TabPosition::EtherCAT,
                        TabPosition::EtherCAT => TabPosition::Modbus,
                        TabPosition::Modbus => TabPosition::Logs,
                        TabPosition::Logs => TabPosition::Logs,
                    }
                }
                VerticalPosition::Page => {
                    self.selected_page().left();
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn handle_init_event(&mut self, event: CrossbeamRuntimeInitEvent) {
        match event {
            CrossbeamRuntimeInitEvent::EtherCATStateUpdate(_) => {}
            CrossbeamRuntimeInitEvent::EtherCATFinalizing => {}
            CrossbeamRuntimeInitEvent::EtherCATDiscoveryStarted => {}
            CrossbeamRuntimeInitEvent::EtherCATDiscoveryCompleted { .. } => {}
            CrossbeamRuntimeInitEvent::EtherCATInitializationStarted => {}
            CrossbeamRuntimeInitEvent::EtherCATDeviceInitializationFailed { .. } => {}
            CrossbeamRuntimeInitEvent::EtherCATDeviceInitializationCompleted { .. } => {}
            CrossbeamRuntimeInitEvent::BuildingMachines => {}
            CrossbeamRuntimeInitEvent::BuiltMachine { ident } => {
                self.page_machines.add_machine(ident);
            }
            CrossbeamRuntimeInitEvent::FailedToBuildMachine { .. } => {}
            _ => {}
        }
    }

    pub fn handle_report(&mut self, report: RuntimeReport) {
        self.page_machines.handle_report(report.machines);
    }
}
