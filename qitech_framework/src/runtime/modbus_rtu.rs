use std::collections::HashMap;
use std::fs;

use qitech_framework_core::modbus::ModbusRTUDeviceMetadata;
use qitech_framework_core::modbus::ModbusRtuAssignment;

const BY_PATH: &str = "/dev/serial/by-path";
const BY_ID: &str = "/dev/serial/by-id";

/// One USB serial port as physically discovered, before joining against any assignment.
struct DiscoveredPort {
    /// `/dev/serial/by-path` basename - the stable key we persist assignments under.
    port: String,
    device_node: String,
    by_id: Option<String>,
    usb_vid: Option<u16>,
    usb_pid: Option<u16>,
    usb_serial: Option<String>,
    description: Option<String>,
}

/// Enumerate `/dev/serial/by-path` once, canonicalize each entry to its device node, and enrich
/// with USB metadata (vendor/product id, serial number, description) and the `/dev/serial/by-id`
/// alias when available.
///
/// Falls back to `serialport::available_ports()` keyed by `port_name` when `/dev/serial/by-path`
/// doesn't exist (e.g. macOS, or a Linux box with no USB serial adapter plugged in), so the Setup
/// page isn't empty during development. Ports found this way have no stable `by-path` id, so the
/// device node itself is used as the port key - it will not survive replugging into a different
/// USB port, but it is the best available on a system without `by-path`.
struct UsbMeta {
    vid: u16,
    pid: u16,
    serial: Option<String>,
    description: Option<String>,
}

fn discover_ports() -> Vec<DiscoveredPort> {
    let usb_by_node: HashMap<String, UsbMeta> = serialport::available_ports()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|p| {
            let serialport::SerialPortType::UsbPort(usb) = p.port_type else {
                return None;
            };
            let description = usb.product.clone().or_else(|| usb.manufacturer.clone());
            Some((
                p.port_name,
                UsbMeta {
                    vid: usb.vid,
                    pid: usb.pid,
                    serial: usb.serial_number,
                    description,
                },
            ))
        })
        .collect();

    let by_id_by_node: HashMap<String, String> = fs::read_dir(BY_ID)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let node = fs::canonicalize(entry.path()).ok()?;
            Some((
                node.to_string_lossy().to_string(),
                entry.file_name().to_string_lossy().to_string(),
            ))
        })
        .collect();

    let by_path_entries: Vec<(String, String)> = fs::read_dir(BY_PATH)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let node = fs::canonicalize(entry.path()).ok()?;
            Some((
                entry.file_name().to_string_lossy().to_string(),
                node.to_string_lossy().to_string(),
            ))
        })
        .collect();

    let ports: Vec<(String, String)> = if !by_path_entries.is_empty() {
        by_path_entries
    } else {
        let mut by_suffix: HashMap<&str, &str> = HashMap::new();

        for node in usb_by_node.keys() {
            let basename = node.rsplit('/').next().unwrap_or(node);
            let Some(suffix) = basename
                .strip_prefix("cu.")
                .or_else(|| basename.strip_prefix("tty."))
            else {
                by_suffix.insert(node, node);
                continue;
            };

            by_suffix
                .entry(suffix)
                .and_modify(|existing| {
                    if node.contains("/cu.") || node.starts_with("cu.") {
                        *existing = node;
                    }
                })
                .or_insert(node);
        }

        by_suffix
            .into_values()
            .map(|node| (node.to_string(), node.to_string()))
            .collect()
    };

    ports
        .into_iter()
        .map(|(port, device_node)| {
            let usb = usb_by_node.get(&device_node);

            DiscoveredPort {
                port,
                by_id: by_id_by_node.get(&device_node).cloned(),
                usb_vid: usb.map(|u| u.vid),
                usb_pid: usb.map(|u| u.pid),
                usb_serial: usb.and_then(|u| u.serial.clone()),
                description: usb.and_then(|u| u.description.clone()),
                device_node,
            }
        })
        .collect()
}

/// Resolve a `/dev/serial/by-path` binding to its current device node, if the port is plugged in.
pub fn resolve_serial_by_path(binding: &str) -> Option<String> {
    discover_ports()
        .into_iter()
        .find(|p| p.port == binding)
        .map(|p| p.device_node)
}

/// The union of physically discovered ports and stored assignments: a port that is assigned but
/// currently unplugged still shows up (`present: false`) so it can be reassigned or unassigned
/// from the UI.
pub fn list_modbus_devices() -> Vec<ModbusRTUDeviceMetadata> {
    let mut assignments: HashMap<String, ModbusRtuAssignment> = read_assignments()
        .into_iter()
        .map(|a| (a.port.clone(), a))
        .collect();

    let mut devices: Vec<ModbusRTUDeviceMetadata> = discover_ports()
        .into_iter()
        .map(|p| ModbusRTUDeviceMetadata {
            assignment: assignments.remove(&p.port),
            port: p.port,
            present: true,
            device_node: Some(p.device_node),
            by_id: p.by_id,
            description: p.description,
            usb_vid: p.usb_vid,
            usb_pid: p.usb_pid,
            usb_serial: p.usb_serial,
        })
        .collect();

    // --- assignments left over have no matching physical port right now ---
    devices.extend(assignments.into_values().map(|a| ModbusRTUDeviceMetadata {
        port: a.port.clone(),
        present: false,
        device_node: None,
        by_id: None,
        description: None,
        usb_vid: None,
        usb_pid: None,
        usb_serial: None,
        assignment: Some(a),
    }));

    devices
}

pub fn read_assignments() -> Vec<ModbusRtuAssignment> {
    let path = config_path();

    let Ok(json) = fs::read_to_string(&path) else {
        return Vec::new();
    };

    serde_json::from_str(&json).unwrap_or_else(|e| {
        tracing::error!("failed to parse modbus rtu assignments at {path}: {e}");
        Vec::new()
    })
}

pub fn write_assignment(assignment: ModbusRtuAssignment) -> std::io::Result<()> {
    let mut assignments = read_assignments();

    if let Some(existing) = assignments.iter_mut().find(|a| a.port == assignment.port) {
        *existing = assignment;
    } else {
        assignments.push(assignment);
    }

    save_assignments(&assignments)
}

pub fn remove_assignment(port: &str) -> std::io::Result<()> {
    let mut assignments = read_assignments();
    assignments.retain(|a| a.port != port);
    save_assignments(&assignments)
}

fn save_assignments(assignments: &[ModbusRtuAssignment]) -> std::io::Result<()> {
    let path = config_path();
    let tmp_path = format!("{path}.tmp");

    let json = serde_json::to_string_pretty(assignments)
        .expect("ModbusRtuAssignment is always serializable");

    fs::write(&tmp_path, json)?;
    fs::rename(&tmp_path, &path)?;

    Ok(())
}

fn config_path() -> String {
    let dir = std::env::var("STATE_DIRECTORY")
        .or(std::env::var("XDG_DATA_HOME"))
        .or(std::env::var("HOME"))
        .unwrap_or(".".to_string());

    dir + "/qitech_modbus.json"
}
