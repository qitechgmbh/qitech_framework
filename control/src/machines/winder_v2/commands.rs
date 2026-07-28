use qitech_framework::machine::error::CommandExecuteResult;

use crate::machines::{WinderV2, winder_v2::{LASER_PORT, TRAVERSE_PORT, TraverseMode, types::Mode}};

pub struct Commands {

}

// --- commands ---
impl WinderV2 {
    pub fn enable_traverse_laserpointer(&mut self, value: bool) -> CommandExecuteResult {
        self.laser_enabled.set(value);
        self.laser.borrow_mut().set_output(LASER_PORT, value);
        Ok(())
    }

    pub fn set_mode(&mut self, mode: Mode) -> CommandExecuteResult {
        let should_update = mode != Mode::Wind || self.can_wind();

        if should_update {
            // all transitions are allowed
            self.mode.set(mode);

            // Apply the mode changes to the spool and puller
            self.set_spool_mode(mode);
            self.set_puller_mode(mode);
            self.set_traverse_mode(mode);
        }

        Ok(())
    }
}

// --- commands utils
impl WinderV2 {
    /// Apply the mode changes to the spool
    ///
    /// It contains a transition matrix for atomic changes.
    /// It will set [`Self::spool_mode`]
    fn set_traverse_mode(&mut self, mode: Mode) {
        // Convert to `Winder2Mode` to `TraverseMode`
        let mode: TraverseMode = mode.clone().into();
        // If coming out of standby
        if self.traverse_mode == TraverseMode::Standby && mode != TraverseMode::Standby {
            let mut traverse = self.traverse.borrow_mut();
            let traverse_ref = &mut *traverse;
            traverse_ref.set_enabled(TRAVERSE_PORT, true);
            self.traverse_controller.set_enabled(true);
            drop(traverse);
        }

        // If going into standby
        if mode == TraverseMode::Standby && self.traverse_mode != TraverseMode::Standby {
            let mut traverse = self.traverse.borrow_mut();
            let traverse_ref = &mut *traverse;
            // If we are going into standby, we need to stop the traverse
            traverse_ref.set_enabled(TRAVERSE_PORT, false);
            self.traverse_controller.set_enabled(false);
            drop(traverse);
        }

        {
            let mut traverse = self.traverse.borrow_mut();
            let traverse_ref = &mut *traverse;
            // Transition matrix
            match self.traverse_mode {
                TraverseMode::Standby => match mode {
                    TraverseMode::Standby => {}
                    TraverseMode::Hold => {
                        // From [`TraverseMode::Standby`] to [`TraverseMode::Hold`]
                        traverse_ref.set_enabled(TRAVERSE_PORT, true);
                        self.traverse_controller.set_enabled(true);
                        self.traverse_controller.goto_home();
                    }
                    TraverseMode::Traverse => {
                        // From [`TraverseMode::Standby`] to [`TraverseMode::Wind`]
                        traverse_ref.set_enabled(TRAVERSE_PORT, true);
                        self.traverse_controller.set_enabled(true);
                        self.traverse_controller.start_traversing();
                    }
                },
                TraverseMode::Hold => match mode {
                    TraverseMode::Standby => {
                        // From [`TraverseMode::Hold`] to [`TraverseMode::Standby`]
                        traverse_ref.set_enabled(TRAVERSE_PORT, false);
                        self.traverse_controller.set_enabled(false);
                    }
                    TraverseMode::Hold => {}
                    TraverseMode::Traverse => {
                        // From [`TraverseMode::Hold`] to [`TraverseMode::Wind`]
                        self.traverse_controller.start_traversing();
                    }
                },
                TraverseMode::Traverse => match mode {
                    TraverseMode::Standby => {
                        // From [`TraverseMode::Wind`] to [`TraverseMode::Standby`]
                        traverse_ref.set_enabled(TRAVERSE_PORT, false);
                        self.traverse_controller.set_enabled(false);
                    }
                    TraverseMode::Hold => {
                        // From [`TraverseMode::Wind`] to [`TraverseMode::Hold`]
                        self.traverse_controller.goto_home();
                    }
                    TraverseMode::Traverse => {}
                },
            }
        }

        // Update the internal state
        self.traverse_mode = mode;
        self.emit_state();
    }
}
