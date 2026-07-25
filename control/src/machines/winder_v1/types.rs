use qitech_framework::machine::resource::CommandHandle;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Standby,
    Hold,
    Pull,
    Wind,
}

pub struct Commands {
    enter_standby: CommandHandle,
    enter_hold: CommandHandle,
    start_pulling: CommandHandle,
    start_winding: CommandHandle,

    traverse_go_home: CommandHandle,
    goto_limit_outer: CommandHandle,
    goto_limit_inner: CommandHandle,

    spool_auto_stop_reset_progress: CommandHandle,

    laser_enable: CommandHandle,
    laser_disable: CommandHandle,
}
