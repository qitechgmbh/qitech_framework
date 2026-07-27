use qitech_framework::machine::CommandHandle;
use qitech_framework::machine::error::CommandExecuteResult;

use crate::machines::WinderV1;
use crate::machines::winder_v1::types::Mode;

pub struct Commands {
    pub enter_standby: CommandHandle,
    pub enter_hold: CommandHandle,
    pub start_pulling: CommandHandle,
    pub start_winding: CommandHandle,

    pub traverse_goto_home: CommandHandle,
    pub traverse_goto_limit_outer: CommandHandle,
    pub traverse_goto_limit_inner: CommandHandle,

    pub spool_auto_stop_reset_progress: CommandHandle,

    pub laser_enable: CommandHandle,
    pub laser_disable: CommandHandle,
}

impl WinderV1 {
    pub fn enter_standby(&mut self) -> CommandExecuteResult {
        self.commands.enter_standby.set_enabled(false);
        self.mode.set(Mode::Standby);
        Ok(())
    }

    pub fn enter_hold(&mut self) -> CommandExecuteResult {
        self.mode.set(Mode::Hold);
        Ok(())
    }

    pub fn start_pulling(&mut self) -> CommandExecuteResult {
        self.mode.set(Mode::Pull);
        Ok(())
    }

    pub fn start_winding(&mut self) -> CommandExecuteResult {
        self.mode.set(Mode::Wind);
        Ok(())
    }

    pub fn laser_enable(&mut self) -> CommandExecuteResult {
        // self.travserse.enable_laser();
        Ok(())
    }

    pub fn disable_laser(&mut self) -> CommandExecuteResult {
        // self.travserse.disable_laser();
        Ok(())
    }

    pub fn traverse_goto_limit_inner(&mut self) -> CommandExecuteResult {
        // self.travserse.goto_limit_inner();
        Ok(())
    }

    pub fn traverse_goto_limit_outer(&mut self) -> CommandExecuteResult {
        // self.travserse.goto_limit_outer();
        Ok(())
    }

    pub fn traverse_goto_home(&mut self) -> CommandExecuteResult {
        // self.travserse.goto_home();
        Ok(())
    }

    pub fn spool_auto_stop_reset_progress(&mut self) -> CommandExecuteResult {
        Ok(())
    }
}
