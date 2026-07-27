use std::time::Duration;

use qitech_lib::ethercat_hal::MasterConfiguration;

pub enum EtherCATMode {
    Disabled,
    Enabled(EtherCATConfig),

    #[allow(unused)]
    Mock,
}

pub struct EtherCATConfig {
    pub interface_scan_interval: Duration,
    pub master_config: Option<MasterConfiguration>,
    pub stay_in_preop: bool,
}

pub enum ModbusMode {
    Disabled,
    Enabled,

    #[allow(unused)]
    Mock,
}
