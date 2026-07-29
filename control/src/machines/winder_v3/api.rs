mod winder2_imports {
    pub use std::time::Instant;

    pub use serde::Deserialize;
    pub use serde::Serialize;

    pub use super::super::Winder2;
    pub use super::super::Winder2Mode;
    pub use super::super::puller_speed_controller::GearRatio;
    pub use super::super::puller_speed_controller::PullerRegulationMode;
}

use qitech_framework::machine::error::CommandExecuteResult;
use qitech_framework::machine::resource::Measurement;
use qitech_framework::machine::resource::StateProperty;
use qitech_lib::units::Angle;
use qitech_lib::units::AngularVelocity;
use qitech_lib::units::Length;
use qitech_lib::units::Velocity;
pub use winder2_imports::*;

use crate::machines::winder_v3::spool_speed_controller::SpoolSpeedControllerType;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum Mode {
    #[default]
    Standby,
    Hold,
    Pull,
    Wind,
}

impl From<Winder2Mode> for Mode {
    fn from(mode: Winder2Mode) -> Self {
        match mode {
            Winder2Mode::Standby => Self::Standby,
            Winder2Mode::Hold => Self::Hold,
            Winder2Mode::Pull => Self::Pull,
            Winder2Mode::Wind => Self::Wind,
        }
    }
}

impl From<Mode> for Winder2Mode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Standby => Self::Standby,
            Mode::Hold => Self::Hold,
            Mode::Pull => Self::Pull,
            Mode::Wind => Self::Wind,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub enum Mutation {
    // Traverse
    /// Position in mm from home point
    SetTraverseLimitOuter(f64),
    /// Position in mm from home point
    SetTraverseLimitInner(f64),
    /// Step size in mm for traverse movement
    SetTraverseStepSize(f64),
    /// Padding in mm for traverse movement limits
    SetTraversePadding(f64),
    GotoTraverseLimitOuter,
    GotoTraverseLimitInner,
    /// Find home point
    GotoTraverseHome,
    EnableTraverseLaserpointer(bool),

    // Puller
    /// on = speed, off = stop
    SetPullerRegulationMode(PullerRegulationMode),
    SetPullerTargetSpeed(f64),
    SetPullerTargetDiameter(f64),
    SetPullerForward(bool),
    SetPullerGearRatio(GearRatio),

    // Spool Speed Controller
    SetSpoolRegulationMode(super::spool_speed_controller::SpoolSpeedControllerType),
    SetSpoolMinMaxMinSpeed(f64),
    SetSpoolMinMaxMaxSpeed(f64),
    SetSpoolForward(bool),

    // Adaptive Spool Speed Controller Parameters
    SetSpoolAdaptiveTensionTarget(f64),
    SetSpoolAdaptiveRadiusLearningRate(f64),
    SetSpoolAdaptiveMaxSpeedMultiplier(f64),
    SetSpoolAdaptiveAccelerationFactor(f64),
    SetSpoolAdaptiveDeaccelerationUrgencyMultiplier(f64),

    // Spool Auto Stop/Pull
    SetSpoolAutomaticRequiredMeters(f64),
    SetSpoolAutomaticAction(SpoolAutomaticActionMode),
    ResetSpoolProgress,

    // Tension Arm
    ZeroTensionArmAngle,

    // Mode
    SetMode(Mode),

    // Set puller adaptive reference machine
    /// Maximum speed change as a percentage of base speed (0.0–100.0)
    SetPullerAdaptiveMaxSpeedChangePercent(f64),
    /// Minimum meters between consecutive adjustments
    SetPullerAdaptiveAdjustmentIntervalMeters(f64),
    /// Step size per adjustment as a percentage of base speed (0.0–100.0)
    SetPullerAdaptiveStepPercent(f64),
    /// Inner deadzone: max deviation from target (mm) that requires no correction
    SetPullerAdaptiveAcceptedDifference(f64),
}

pub struct Measurements {
    /// traverse position in mm
    pub traverse_position: Measurement<Option<Length>>,
    /// puller speed in m/min
    pub puller_speed: Measurement<Velocity>,
    /// spool rpm
    pub spool_rpm: Measurement<AngularVelocity>,
    /// tension arm angle in degrees
    pub tension_arm_angle: Measurement<Angle>,
    // spool progress in meters (pulled distance of filament)
    pub spool_progress: Measurement<Length>,
}

pub struct States {
    /// traverse state
    pub traverse_state: TraverseState,
    /// puller state
    pub puller_state: PullerState,
    /// spool automatic action state and progress
    pub spool_automatic_action_state: SpoolAutomaticActionState,
    /// mode state
    pub mode_state: ModeState,
    /// tension arm state
    pub tension_arm_state: TensionArmState,
    /// spool speed controller state
    pub spool_speed_controller_state: SpoolSpeedControllerState,
    /// Is a Machine Connected?
    pub puller_reference_machine: StateProperty<Option<u64>>,
}

pub struct TraverseState {
    /// min position in mm
    pub limit_inner: StateProperty<Length>,
    /// max position in mm
    pub limit_outer: StateProperty<Length>,
    /// is going to position in
    pub is_going_in: StateProperty<bool>,
    /// is going to position out
    pub is_going_out: StateProperty<bool>,
    /// if is homed
    pub is_homed: StateProperty<bool>,
    /// if is homing
    pub is_going_home: StateProperty<bool>,
    /// if is traversing
    pub is_traversing: StateProperty<bool>,
    /// laserpointer is on
    pub laserpointer: StateProperty<bool>,
    /// step size in mm
    pub step_size: StateProperty<Length>,
    /// padding in mm
    pub padding: StateProperty<Length>,
    /// can go in (to inner limit)
    pub can_go_in: StateProperty<bool>,
    /// can go out (to outer limit)
    pub can_go_out: StateProperty<bool>,
    /// can home
    pub can_go_home: StateProperty<bool>,
}

pub struct PullerState {
    /// regulation type
    pub regulation: StateProperty<PullerRegulationMode>,
    /// target speed in m/min
    pub target_speed: StateProperty<Velocity>,
    /// forward rotation direction
    pub forward: StateProperty<bool>,
    /// gear ratio for winding speed
    pub gear_ratio: StateProperty<GearRatio>,

    /// Maximum speed change as a percentage of base speed (0.0–100.0)
    pub adaptive_speed_delta_max: StateProperty<f64>,
    /// Minimum meters between consecutive adjustments
    pub adaptive_adjustment_distance: StateProperty<Length>,
    /// Step size per adjustment as a percentage of base speed (0.0–100.0)
    pub adaptive_change_per_step: StateProperty<f64>,
    /// Inner deadzone: max deviation from target (mm) that requires no correction
    pub allowed_diameter_deviation: StateProperty<Length>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub enum SpoolAutomaticActionMode {
    #[default]
    NoAction,
    Pull,
    Hold,
}

pub struct SpoolAutomaticActionState {
    pub spool_required_meters: StateProperty<Length>,
    pub spool_automatic_action_mode: SpoolAutomaticActionMode,
}

pub struct ModeState {
    /// mode
    pub mode: Mode,
    /// can wind
    pub can_wind: StateProperty<bool>,
}

pub struct TensionArmState {
    /// is zeroed
    pub zeroed: StateProperty<bool>,
}

pub struct SpoolSpeedControllerState {
    /// regulation mode
    pub regulation_mode: SpoolSpeedControllerType,
    /// min speed in rpm for minmax mode
    pub minmax_min_speed: StateProperty<AngularVelocity>,
    /// max speed in rpm for minmax mode
    pub minmax_max_speed: StateProperty<AngularVelocity>,
    /// tension target for adaptive mode (0.0-1.0)
    pub adaptive_tension_target: StateProperty<f64>,
    /// radius learning rate for adaptive mode
    pub adaptive_radius_learning_rate: StateProperty<f64>,
    /// max speed multiplier for adaptive mode
    pub adaptive_max_speed_multiplier: StateProperty<f64>,
    /// acceleration factor for adaptive mode
    pub adaptive_acceleration_factor: StateProperty<f64>,
    /// deacceleration urgency multiplier for adaptive mode
    pub adaptive_deacceleration_urgency_multiplier: StateProperty<f64>,
    /// forward rotation direction
    pub forward: StateProperty<bool>,
}

impl Winder2 {
    pub fn execute_mutation(&mut self, mutation: Mutation) -> CommandExecuteResult {
        match mutation {
            Mutation::EnableTraverseLaserpointer(enable) => self.set_laser(enable),
            Mutation::SetMode(mode) => self.set_mode(&mode.into()),
            Mutation::SetTraverseLimitOuter(limit) => self.traverse_set_limit_outer(limit),
            Mutation::SetTraverseLimitInner(limit) => self.traverse_set_limit_inner(limit),
            Mutation::SetTraverseStepSize(size) => self.traverse_set_step_size(size),
            Mutation::SetTraversePadding(padding) => self.traverse_set_padding(padding),
            Mutation::GotoTraverseLimitOuter => self.traverse_goto_limit_outer(),
            Mutation::GotoTraverseLimitInner => self.traverse_goto_limit_inner(),
            Mutation::GotoTraverseHome => self.traverse_goto_home(),
            Mutation::SetPullerRegulationMode(regulation) => self.puller_set_regulation(regulation),
            Mutation::SetPullerTargetSpeed(value) => self.puller_set_target_speed(value),
            Mutation::SetPullerTargetDiameter(_) => todo!(),
            Mutation::SetPullerForward(value) => self.puller_set_forward(value),
            Mutation::SetPullerGearRatio(gear_ratio) => self.puller_set_gear_ratio(gear_ratio),
            Mutation::SetSpoolRegulationMode(mode) => self.spool_set_regulation_mode(mode),
            Mutation::SetSpoolMinMaxMinSpeed(speed) => self.spool_set_minmax_min_speed(speed),
            Mutation::SetSpoolMinMaxMaxSpeed(speed) => self.spool_set_minmax_max_speed(speed),
            Mutation::SetSpoolForward(value) => self.spool_set_forward(value),
            Mutation::SetSpoolAdaptiveTensionTarget(value) => {
                self.spool_set_adaptive_tension_target(value)
            }
            Mutation::SetSpoolAdaptiveRadiusLearningRate(value) => {
                self.spool_set_adaptive_radius_learning_rate(value)
            }
            Mutation::SetSpoolAdaptiveMaxSpeedMultiplier(value) => {
                self.spool_set_adaptive_max_speed_multiplier(value)
            }
            Mutation::SetSpoolAdaptiveAccelerationFactor(value) => {
                self.spool_set_adaptive_acceleration_factor(value)
            }
            Mutation::SetSpoolAdaptiveDeaccelerationUrgencyMultiplier(value) => {
                self.spool_set_adaptive_deacceleration_urgency_multiplier(value)
            }
            Mutation::SetSpoolAutomaticRequiredMeters(meters) => {
                self.set_spool_automatic_required_meters(meters)
            }
            Mutation::SetSpoolAutomaticAction(mode) => self.set_spool_automatic_mode(mode),
            Mutation::ResetSpoolProgress => self.stop_or_pull_spool_reset(Instant::now()),
            Mutation::ZeroTensionArmAngle => self.tension_arm_zero(),

            // puller adaptive speed algorithm
            Mutation::SetPullerAdaptiveMaxSpeedChangePercent(v) => {
                self.puller_set_adaptive_max_speed_change_percent(v)
            }
            Mutation::SetPullerAdaptiveAdjustmentIntervalMeters(v) => {
                self.puller_set_adaptive_adjustment_interval_meters(v)
            }
            Mutation::SetPullerAdaptiveStepPercent(v) => self.puller_set_adaptive_step_percent(v),
            Mutation::SetPullerAdaptiveAcceptedDifference(v) => {
                self.puller_set_adaptive_accepted_difference(v)
            }
        }

        Ok(())
    }
}
