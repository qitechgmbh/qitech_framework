# Minimal-Example Digital Output Video-Skript

In diesem Skript zeigen wir wie man das QiTech Framework nutzt um eine Digital-Output EtherCAT klemme anzusteuern.

## Neues Projekt erstellen mit cargo init

## QiTech Framework als dependency

```toml
[dependencies]
qitech_framework = { git = "https://github.com/qitechgmbh/qitech_framework", rev = "53853813d1182f74924ec493d710bc8f75752ea5" }
qitech_framework_tui = { git = "https://github.com/qitechgmbh/qitech_framework", rev = "53853813d1182f74924ec493d710bc8f75752ea5" }
```

## Start QiTech Runtime without any hardware

```rust
let config = RuntimeConfiguration::new();
let session = session::debug::runtime();

let rt = Runtime::init(config, session).expect("Failed to create runtime!");
rt.run().expect("Runtime error!");
```

## Add EtherCAT (and connect hardware)

```rust
let config = RuntimeConfiguration::new()
    .cycle_period(Duration::from_millis(100))
    .ethercat(EtherCATConfig {
        interface_scan_interval: Duration::from_secs(1),
        master_config: None,
        stay_in_preop: false,
    });
```

Einmal ausführen → nichts geht. Dann cargo build und run mit sudo!

## Selbe Config mit TUI

```rust
run_with_tui(config_rt, TuiConfiguration::default())
.await
.unwrap()
```

## Neue Machine erstellen. Hier Beispielhaft den EL2004 Digital Output (LEDs an/aus)

Ziel: Eine LEDs über die TUI an/aus machen.

### Struct für Machine erstellen

```rust
pub struct EL2004Machine {
    leds_on: ConfigProperty,
    el2004: Rc<RefCell>,
}
```

Eine weitere Dependency wird benötigt, für die Beckhoff Klemme

```toml
ethercat_hal = { git = "https://github.com/qitechgmbh/qitech_lib", rev = "530355c00baf6335f6085441c2a497d9ac060af6" }
```

### Machine bei Runtime Config angeben

```rust
let config = RuntimeConfiguration::new()
    .cycle_period(Duration::from_millis(100))
    .ethercat(EtherCATConfig {
        interface_scan_interval: Duration::from_secs(1),
        master_config: None,
        stay_in_preop: false,
    })
    .machine::();
```

Geht nicht, da Traits fehlen, die Implementieren wir jetzt

### Zunächst implementieren wir `Machine`

Leeres act() for now. Machine macht erst mal noch nichts

```rust
fn act(&mut self, dt: Duration) -> qitech_framework::machine::ActResult {
    Ok(())
}
```

### Dann MachineDescriptor. Das sagt, wie unsere Machine einzuordnen ist.

Die IDENTIFICATION gibt an, welche Klemmen für unsere Machine wichtig sind. Standard ist alles 0. QiTech Framework kann diese werte noch nicht setzten, kommt aber bald dazu

```rust
const IDENTIFICATION: MachineIdentification = MachineIdentification {
    vendor_id: 0,
    machine_id: 0,
};
```

Das SCHEMA zeigt auf eine YAML Datei. Hier steht drin, wie QiTech Framework die Daten der Machine nach außen (ua. in der TUI anzeigen soll).

```rust
const SCHEMA: &'static str = include_str!("../EL2004Machine.yml");
```

### Yaml Datei erstellen

```yaml
qms_version: 1.0
revision: 1

identification:
  name: EL2004
  vendor_id: 0
  machine_id: 0

config:
  leds_on: !boolean
```

Config kann der Nutzer mit der TUI direkt schreiben. identification müssen gleich sein.

### Zuletzt noch MachineBuild implementieren

```rust
impl MachineBuild for EL2004Machine {
    fn build(ctx: &mut BuildContext) -> BuildResult<Self> {

    }
}
```

Wir finden unsere Klemme

```rust
let el2004 = ctx.find_ethercat_device::(1)?;
```

Wir hohlen uns den Wert aus dem Config Schema

```rust
let leds_on = ctx
    .config::("leds_on")
    .default(false)
    .build()?;
```

Und geben die Fertige Machine zurück

```rust
Ok(Self {
    el2004,
    leds_on,
})
```

### Zuletzt in act() die Werte überschreiben

```rust
let mut el2004 = self.el2004.borrow_mut();

for port in 0..el2004.get_port_count() {
    el2004.set_output(port, self.leds_on.get());
}

Ok(())
```