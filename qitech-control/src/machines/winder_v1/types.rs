use control_runtime::machine::Command;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Standby,
    Hold,
    Pull,
    Wind,
}

pub struct Commands {
    enter_standby: Command,
    enter_hold:    Command,
    start_pulling: Command,
    start_winding: Command,

    traverse_go_home: Command,
    goto_limit_outer: Command,
    goto_limit_inner: Command,

    spool_auto_stop_reset_progress: Command,

    laser_enable: Command,
    laser_disable: Command,
}