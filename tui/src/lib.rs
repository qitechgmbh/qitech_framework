
use std::collections::HashMap;

use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;
use crossterm::event::KeyCode;
use indexmap::IndexMap;
use qitech_framework::MachineIdentification;
use qitech_framework::MachineIdentificationUnique;
use qitech_framework::ScalarValue;
use qitech_framework_common::MachineSchema;
use qitech_framework_common::RuntimeInitEvent;
use qitech_framework_common::RuntimeReport;
use qitech_framework_common::RuntimeRequest;
use qitech_framework_common::RuntimeRequestKind;
use qitech_framework_common::RuntimeStatus;
use qitech_framework_common::link::HandleTransport;
use qitech_framework_common::link::handle::session;
use qitech_framework_common::schema;
use qitech_framework_common::schema::ConfigPropertyValue;
use qitech_framework_common::schema::MeasurementValue;
use qitech_framework_common::schema::Node;
use qitech_framework_common::schema::NodeKind;
use qitech_framework_common::schema::StatePropertyValue;
use ratatui::Frame;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;

/* 
mod types;
use types::*;
mod run;
use run::run;
mod controls;
mod utils;

mod widgets;
use widgets::StatusDisplay;

use crate::widgets::TabView;

mod pages;
mod app;
*/

enum SessionMessage<T: HandleTransport> {
    Schemas(HashMap<MachineIdentification, MachineSchema>),
    InitEvent(RuntimeInitEvent),
    Finished(session::Running<T>),
    Disconnected,
}

pub enum SessionState<T: HandleTransport> {
    Initializing {
        rx: Receiver<SessionMessage<T>>,
    },
    Running(session::Running<T>),
    Disconnected,
}
