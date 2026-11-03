pub mod devices;

use std::collections::HashMap;

use tokio::sync::Mutex;
use tokio_modbus::Request;
use tokio_modbus::Response;
use tokio_modbus::Slave;
use tokio_modbus::client::rtu;
use tokio_modbus::client::tcp;
use tokio_serial::SerialPortBuilder;
use tokio_serial::SerialStream;

struct DataBits(tokio_serial::DataBits);
struct StopBits(tokio_serial::StopBits);

pub type ModbusRequest = Request<'static>;
pub type ModbusResponse = Response;
pub type ClientContext = tokio_modbus::client::Context;

pub struct SerialDeviceManager {
    pub port_contexts: HashMap<String, Mutex<ClientContext>>,
    // keep track of devices
    pub devices: HashMap<String, SerialDeviceMeta>,
}

#[derive(Debug, Clone, Copy)]
pub struct ModbusSettings {
    pub baudrate: u32,
    pub bits: u8,
    pub stop_bits: u8,
    pub parity: Parity,
    pub modbus_type: ModbusType,
}

/*
    This trait is suited to applications of Modbus where you
    continually poll the device with the same request over and over
*/
pub trait ModbusDevice {
    fn new(
        path: String,
        slave_id: u8,
        settings: Option<ModbusSettings>,
    ) -> Result<Self, anyhow::Error>
    where
        Self: Sized;
    // When send_request fails device is dead and should be dropped
    // Prepare your request to be sent with whatever logic you wish
    fn send_next_request(&mut self) -> Result<(), anyhow::Error>;
    fn handle_response(&mut self) -> Result<(), anyhow::Error>;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub fn create_modbus_device_context(
    dev: &SerialDeviceMeta,
) -> Result<ClientContext, anyhow::Error> {
    let device_builder = create_modbus_builder(dev)?;
    let port = SerialStream::open(&device_builder)?;
    let slave = Slave(dev.slave_id);
    match dev.modbus_type {
        ModbusType::Rtu => Ok(rtu::attach_slave(port, slave)),
        ModbusType::Tcp => Ok(tcp::attach_slave(port, slave)),
    }
}

fn create_modbus_builder(dev: &SerialDeviceMeta) -> Result<SerialPortBuilder, anyhow::Error> {
    let data_bits: DataBits = dev.bits.try_into()?;
    let stop_bits: StopBits = dev.stop_bits.try_into()?;
    Ok(tokio_serial::new(dev.path.clone(), dev.baudrate)
        .flow_control(tokio_serial::FlowControl::None)
        .data_bits(data_bits.0)
        .stop_bits(stop_bits.0)
        .parity(dev.parity.into()))
}

impl TryFrom<u8> for DataBits {
    type Error = anyhow::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            5 => Ok(DataBits(tokio_serial::DataBits::Five)),
            6 => Ok(DataBits(tokio_serial::DataBits::Six)),
            7 => Ok(DataBits(tokio_serial::DataBits::Seven)),
            8 => Ok(DataBits(tokio_serial::DataBits::Eight)),
            _ => Err(anyhow::anyhow!("DataBits specified not Valid")),
        }
    }
}

impl TryFrom<u8> for StopBits {
    type Error = anyhow::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(StopBits(tokio_serial::StopBits::One)),
            2 => Ok(StopBits(tokio_serial::StopBits::Two)),
            _ => Err(anyhow::anyhow!("StopBits specified not Valid")),
        }
    }
}

impl From<Parity> for tokio_serial::Parity {
    fn from(value: Parity) -> Self {
        match value {
            Parity::None => tokio_serial::Parity::None,
            Parity::Even => tokio_serial::Parity::Even,
            Parity::Odd => tokio_serial::Parity::Odd,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Parity {
    None,
    Even,
    Odd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModbusType {
    Rtu,
    Tcp,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SerialDeviceMeta {
    pub path: String, // can be IP or Fs Path
    pub device_name: Option<String>,
    pub slave_id: u8,
    pub baudrate: u32,
    pub bits: u8,
    pub stop_bits: u8,
    pub parity: Parity,
    pub modbus_type: ModbusType,
}
