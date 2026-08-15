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
    SetPullerAdaptiveReferenceMachine(Option<QiTechMachineIdentificationUnique>),
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {
    /// traverse position in mm
    pub traverse_position: Option<f64>,
    /// puller speed in m/min
    pub puller_speed: f64,
    /// spool rpm
    pub spool_rpm: f64,
    /// tension arm angle in degrees
    pub tension_arm_angle: f64,
    // spool progress in meters (pulled distance of filament)
    pub spool_progress: f64,
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub is_default_state: bool,
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
    pub puller_reference_machine: Option<QiTechMachineIdentificationUnique>,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct TraverseState {
    /// min position in mm
    pub limit_inner: f64,
    /// max position in mm
    pub limit_outer: f64,
    /// position in mm
    pub position_in: f64,
    /// position out in mm
    pub position_out: f64,
    /// is going to position in
    pub is_going_in: bool,
    /// is going to position out
    pub is_going_out: bool,
    /// if is homed
    pub is_homed: bool,
    /// if is homing
    pub is_going_home: bool,
    /// if is traversing
    pub is_traversing: bool,
    /// laserpointer is on
    pub laserpointer: bool,
    /// step size in mm
    pub step_size: f64,
    /// padding in mm
    pub padding: f64,
    /// can go in (to inner limit)
    pub can_go_in: bool,
    /// can go out (to outer limit)
    pub can_go_out: bool,
    /// can home
    pub can_go_home: bool,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct PullerState {
    /// regulation type
    pub regulation: PullerRegulationMode,
    /// target speed in m/min
    pub target_speed: f64,
    /// forward rotation direction
    pub forward: bool,
    /// gear ratio for winding speed
    pub gear_ratio: GearRatio,

    /// Maximum speed change as a percentage of base speed (0.0–100.0)
    pub adaptive_speed_delta_max: f64,
    /// Minimum meters between consecutive adjustments
    pub adaptive_adjustment_distance: f64,
    /// Step size per adjustment as a percentage of base speed (0.0–100.0)
    pub adaptive_change_per_step: f64,
    /// Inner deadzone: max deviation from target (mm) that requires no correction
    pub allowed_diameter_deviation: f64,
    pub adaptive_reference_machine: Option<QiTechMachineIdentificationUnique>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub enum SpoolAutomaticActionMode {
    #[default]
    NoAction,
    Pull,
    Hold,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct SpoolAutomaticActionState {
    pub spool_required_meters: f64,
    pub spool_automatic_action_mode: SpoolAutomaticActionMode,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct ModeState {
    /// mode
    pub mode: Mode,
    /// can wind
    pub can_wind: bool,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct TensionArmState {
    /// is zeroed
    pub zeroed: bool,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct SpoolSpeedControllerState {
    /// regulation mode
    pub regulation_mode: super::spool_speed_controller::SpoolSpeedControllerType,
    /// min speed in rpm for minmax mode
    pub minmax_min_speed: f64,
    /// max speed in rpm for minmax mode
    pub minmax_max_speed: f64,
    /// tension target for adaptive mode (0.0-1.0)
    pub adaptive_tension_target: f64,
    /// radius learning rate for adaptive mode
    pub adaptive_radius_learning_rate: f64,
    /// max speed multiplier for adaptive mode
    pub adaptive_max_speed_multiplier: f64,
    /// acceleration factor for adaptive mode
    pub adaptive_acceleration_factor: f64,
    /// deacceleration urgency multiplier for adaptive mode
    pub adaptive_deacceleration_urgency_multiplier: f64,
    /// forward rotation direction
    pub forward: bool,
}