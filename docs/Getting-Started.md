# Creating a simple example for a modbus rtu device

## 1. Define the Cargo.toml inside your project root

```toml
[package]
name = "example"
version = "0.1.0"
edition = "2024"

[dependencies]
qitech_lib = { git = "https://github.com/qitechgmbh/qitech_lib.git" }
qitech_framework = { git = "https://github.com/qitechgmbh/qitech_framework.git", package = "qitech_framework" }
qitech_framework_tui = { git = "https://github.com/qitechgmbh/qitech_framework.git", package = "qitech_framework_tui" }
```
## 2. Create the main.rs and your runtime configuration

```rust
use qitech_framework::RuntimeConfiguration;

pub fn main() {
    let config = RuntimeConfiguration::new();
}
```

## 3. Determine the USB device path for your usb connected modbus rtu device

```bash
ls -l /dev/serial/by-path/
```

example output:
pci-0000:00:14.0-usb-0:2:1.0-port0 -> ../../ttyUSB0


## 4. Add device to configuration

```rust
use qitech_framework::RuntimeConfiguration;
use qitech_lib::modbus::devices::qitech_laser::LaserDevice;

pub fn main() {
    let config = RuntimeConfiguration::new()
    .modbus_rtu_device::<LaserDevice>(
        "pci-0000:c6:00.0-usbv2-0:2.1:1.0-port0",
        MyMachine::IDENTIFICATION.into_unique(1),
        1,
        None,
    );
}
```

## 5. Creating a new Machine

### 5.1 Defining the Schema

We will be creating a simple Machine for our Laser.

For this we will define one measurement, the measured diameter

2 Config Properties, the target diameter and allowed deviation for the target

1 State Property that tracks if the measured diameter is inside the target range

Inside the root directory where the Cargo.toml lives create a new directory named /schemas and create my_machine.yaml inside it with following contents:

```yaml
qms_version: 1.0
revision: 1

identification:
  name: my_machine
  vendor_id: 0
  machine_id: 1

config:
  diameter:
    target: !millimeter
    tolerance: !millimeter

state:
  in_tolerance: !boolean

measurements:
  diameter: !millimeter
```

Every schema file must contain the qms_version which tracks the schema parser/interpreter version, the revision, which tracks that revision your machines schema is at, meaning if you added a new item/renamed or removed one you would increment this counter so other systems can match it against their currently known one. 

Next is the identification consisting of your machines slug/name the vendor id (0 for none)
and machine id to distinguish your machines from each other.

then we have 3 sections config, state and measurements where we can actually declare the properties we will be using.

### 5.2 Creating the Machine

```rust
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::StateProperty;
use qitech_framework::machine::Measurement;

#[derive(Machine)]
pub struct MyMachine {
    device: Rc<RefCell<LaserDevice>>,

    target: ConfigProperty<Length>,
    tolerance: ConfigProperty<Length>,
    
    in_tolerance: StateProperty<bool>,

    diameter: Measurement<Length>,

    last_request: Instant,
}
```

### 5.3 Building the Machine

```rust
impl MachineBuild for MyMachine {
    #[machine_build(MyMachine)]
    fn build(ctx: &mut BuildContext) -> BuildResult<Self> {
        let device = ctx.get_modbus_rtu_device::<LaserDevice>(0)?;

        let target = ctx
            .config::<millimeter>("diameter.target")
            .default(1.75)
            .register()?;

        let tolerance = ctx
            .config::<millimeter>("diameter.tolerance")
            .default(0.05)
            .register()?;

        let in_tolerance = ctx
            .state::<bool>("in_tolerance")
            .register()?;

        let diameter = ctx
            .measurement::<millimeter>("diameter")
            .register()?;

        Ok(Self {
            device,
            target,
            tolerance,
            in_tolerance,
            diameter,
            last_request: Instant::now(),
        })
    }
}
```

### 5.4 Implmementing Machine for MyMachine

```rust
impl Machine for LaserV1 {
    fn act(&mut self, now: Instant) -> ActResult {
        self.update_device();
        self.update_in_tolerance();

        if let Some(m) = self.device.borrow().measurement.clone() {
            let diameter = Length::new::<millimeter>(value as f64 / 1000.0);
            self.diameter.set(diameter);
        }

        Ok(())
    }
}

impl MyMachine {
    fn update_device(&mut self, now: Instant) {
        let mut laser = self.device.borrow_mut();

        // process the response if any
        _ = laser.handle_response();

        // send request every 6 ms
        if now.duration_since(self.last_request) > Duration::from_millis(6) {
            _ = laser.send_next_request();
            self.last_request = now;
        }
    }

    fn update_in_tolerance(&mut self) {
        let target = self.target.get();
        let top = target + self.tolerance.get();
        let bottom = target - self.tolerance.get();

        let value = self.diameter.get() < top && self.diameter.get() > bottom;
        self.in_tolerance.set(value);
    }
}
```

### 5.5 Registering the Machine into the configuration

```rust
use std::thread;
use std::time::Duration;

use qitech_framework::Runtime;
use qitech_framework::RuntimeConfiguration;
use qitech_framework::session;
use qitech_framework_tui::Tui;
use qitech_framework_tui::TuiConfiguration;
use qitech_lib::modbus::devices::qitech_laser::LaserDevice;

pub fn main() {
    let config = RuntimeConfiguration::new()
    .modbus_rtu_device::<LaserDevice>(
        "pci-0000:c6:00.0-usbv2-0:2.1:1.0-port0",
        LaserV1::IDENTIFICATION.into_unique(1),
        1,
        None,
    )
    .machine::<LaserV1>();
}
```

## 6. Putting it all together

```rust
use std::time::Duration;

use qitech_framework::Runtime;
use qitech_framework::RuntimeConfiguration;
use qitech_framework_tui::App;
use qitech_framework_tui::TuiConfiguration;
use qitech_lib::modbus::devices::qitech_laser::LaserDevice;

pub fn main() {
    let rt_config = RuntimeConfiguration::new()
    .modbus_rtu_device::<LaserDevice>(
        "pci-0000:c6:00.0-usbv2-0:2.1:1.0-port0",
        LaserV1::IDENTIFICATION.into_unique(1),
        1,
        None,
    )
    .machine::<LaserV1>();

    let tui_config = TuiConfiguration::new()
        .refresh_rate(Duration::from_secs_f64(1.0 / 32.0));

    App::launch(tui_config, rt_config)
}
```