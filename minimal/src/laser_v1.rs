use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

use qitech_framework::__private::ConstraintViolationError;
use qitech_framework::__private::Constraints;
use qitech_framework::__private::ScalarValue;
use qitech_framework::__private::ScalarValueTypeMismatchError;
use qitech_framework::machine::MachineDescriptor;
use qitech_framework::prelude::*;
use qitech_framework::resource::EnumConstraints;
use qitech_framework::resource::conversion::PropertyAdapter;
use qitech_framework::resource::conversion::PropertyType;
use qitech_lib::modbus::ModbusDevice;
use qitech_lib::modbus::devices::qitech_laser::LaserDevice;
use qitech_lib::modbus::devices::qitech_laser::LaserError;

#[derive(Debug, Default, Clone, PartialEq)]
pub enum MyEnum {
    #[default]
    Hello,
    World,
}

// --- non optional ---
impl PropertyAdapter for MyEnum {
    type Type = MyEnum;
    type Input = MyEnum;

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::Enum(Some(
            match value {
                MyEnum::Hello => "Hello",
                MyEnum::World => "World",
            }
            .to_string(),
        ))
    }

    fn from_scalar(value: ScalarValue) -> Result<Self::Type, ScalarValueTypeMismatchError> {
        match value {
            ScalarValue::Enum(Some(v)) => match v.as_str() {
                "hello" => Ok(MyEnum::Hello),
                "world" => Ok(MyEnum::World),
                _ => Err(ScalarValueTypeMismatchError),
            },

            _ => Err(ScalarValueTypeMismatchError),
        }
    }

    fn validate_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
        value: &Self::Type,
    ) -> Result<(), ConstraintViolationError> {
        if constraints.allowed.contains(value) {
            Ok(())
        } else {
            let value = Self::into_scalar(value.clone());
            Err(ConstraintViolationError::ForbiddenVariant { value })
        }
    }

    fn as_parameter_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
    ) -> Constraints {
        let mut allowed = Vec::new();

        for variant in constraints.allowed.clone() {
            let value = Self::into_scalar(variant);
            allowed.push(value);
        }

        Constraints::Enum {
            allowed,
            nullable: false,
        }
    }
}

impl PropertyType for MyEnum {
    type Constraints = EnumConstraints<MyEnum>;
}

pub struct LaserV1Subscription {
    ident: MachineIdentificationUnique,

    // --- config ---
    diameter_target: RemoteProperty<Length>,
    diameter_tolerance_upper: RemoteProperty<Length>,
    diameter_tolerance_lower: RemoteProperty<Length>,

    // --- state ----
    in_tolerance: RemoteProperty<bool>,

    // --- measurements ---
    diameter: RemoteProperty<Length>,
    diameter_x: RemoteProperty<Option<Length>>,
    diameter_y: RemoteProperty<Option<Length>>,
    roundness: RemoteProperty<Option<f64>>,
}

// #[machine(schema = "schemas/laser_v1.yaml")]
pub struct LaserV1 {
    // --- hardware ---
    device: Rc<RefCell<LaserDevice>>,

    // -- config ---
    diameter_target: ConfigProperty<Length>,
    diameter_tolerance_upper: ConfigProperty<Length>,
    diameter_tolerance_lower: ConfigProperty<Length>,

    // --- testing ---
    diameter_target_default: ConfigProperty<Length>,
    diameter_target_enabled: ConfigProperty<bool>,
    diameter_target_min: ConfigProperty<Option<Length>>,
    diameter_target_max: ConfigProperty<Option<Length>>,

    subscribed_diameter_target: ConfigProperty<Option<Length>>,
    subscribed_diameter_tolerance_upper: ConfigProperty<Option<Length>>,
    subscribed_diameter_tolerance_lower: ConfigProperty<Option<Length>>,

    // --- state ---
    in_tolerance: StateProperty<bool>,
    subscribed_in_tolerance: StateProperty<Option<bool>>,

    // --- measurements ---
    diameter: Measurement<Length>,
    diameter_x: Measurement<Option<Length>>,
    diameter_y: Measurement<Option<Length>>,
    roundness: Measurement<Option<f64>>,

    // --- subscriptions ---
    subscription: Option<LaserV1Subscription>,

    subscribed_diameter: Measurement<Option<Length>>,
    subscribed_diameter_x: Measurement<Option<Length>>,
    subscribed_diameter_y: Measurement<Option<Length>>,
    subscribed_roundness: Measurement<Option<f64>>,

    // -- misc ---
    last_request: Instant,
}

impl MachineDescriptor for LaserV1 {
    const SCHEMA: &'static str = include_str!("../schemas/laser_v1.yaml");
    const IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor_id: 1,
        machine_id: 16,
    };
}

impl MachineBuild for LaserV1 {
    fn build(ctx: &mut BuildContext) -> BuildResult<Self> {
        let device = ctx.get_modbus_rtu_device::<LaserDevice>(0)?;

        let diameter_target = ctx
            .config::<millimeter>("diameter.target")
            .default(1.75)
            .register()?;

        let diameter_tolerance_lower = ctx
            .config::<millimeter>("diameter.tolerance.lower")
            .default(0.05)
            // .minimum(0.0)
            .register()?;

        let diameter_tolerance_upper = ctx
            .config::<millimeter>("diameter.tolerance.upper")
            .default(0.05)
            // .minimum(0.0)
            .register()?;

        let diameter_target_default = ctx
            .config::<millimeter>("diameter.target_default")
            .default(1.75)
            .on_external_write(|m: &mut Self| {
                let default = *m.diameter_target_default.get_ref();
                m.diameter_target.set_default(default);
            })
            .register()?;

        let diameter_target_enabled = ctx
            .config::<bool>("diameter.target_enabled")
            .default(true)
            .on_external_write(|m: &mut Self| match *m.diameter_target_enabled.get_ref() {
                true => m.diameter_target.allow_external_write(),
                false => m.diameter_target.forbid_external_write("single use only"),
            })
            .register()?;

        let diameter_target_min = ctx
            .config::<Option<millimeter>>("diameter.target_min")
            .default(None)
            .on_external_write(|m: &mut Self| {
                let mut c = m.diameter_target.constraints().clone();
                c.set_min(*m.diameter_target_min.get_ref());
                m.diameter_target.set_constraints(c);
            })
            .register()?;

        let diameter_target_max = ctx
            .config::<Option<millimeter>>("diameter.target_max")
            .default(None)
            .on_external_write(|m: &mut Self| {
                let mut c = m.diameter_target.constraints().clone();
                c.set_max(*m.diameter_target_max.get_ref());
                m.diameter_target.set_constraints(c);
            })
            .register()?;

        let subscribed_diameter_target = ctx
            .config::<Option<millimeter>>("subscribed.diameter.target")
            .default(None)
            .register()?;

        let subscribed_diameter_tolerance_lower = ctx
            .config::<Option<millimeter>>("subscribed.diameter.tolerance.lower")
            .default(None)
            .register()?;

        let subscribed_diameter_tolerance_upper = ctx
            .config::<Option<millimeter>>("subscribed.diameter.tolerance.upper")
            .default(None)
            .register()?;

        // --- state ---
        let in_tolerance = ctx
            .state::<bool>("in_tolerance")
            .initial(false)
            .register()?;

        let subscribed_in_tolerance = ctx
            .state::<Option<bool>>("subscribed.in_tolerance")
            .initial(None)
            .register()?;

        // --- measurements ---
        let subscribed_diameter = ctx
            .measurement::<Option<millimeter>>("subscribed.diameter")
            .register()?;

        let subscribed_diameter_x = ctx
            .measurement::<Option<millimeter>>("subscribed.diameter_x")
            .register()?;

        let subscribed_diameter_y = ctx
            .measurement::<Option<millimeter>>("subscribed.diameter_y")
            .register()?;

        let subscribed_roundness = ctx
            .measurement::<Option<f64>>("subscribed.roundness")
            .register()?;

        let x = ctx.config::<MyEnum>("x").register()?;
        let y = ctx.state::<MyEnum>("y").register()?;

        Ok(Self {
            device,
            diameter_target,
            diameter_tolerance_lower,
            diameter_tolerance_upper,
            diameter_target_default,
            diameter_target_enabled,
            diameter_target_min,
            diameter_target_max,
            subscribed_diameter_target,
            subscribed_diameter_tolerance_lower,
            subscribed_diameter_tolerance_upper,
            in_tolerance,
            subscribed_in_tolerance,
            diameter: ctx.measurement::<millimeter>("diameter").register()?,
            diameter_x: ctx
                .measurement::<Option<millimeter>>("diameter_x")
                .register()?,
            diameter_y: ctx
                .measurement::<Option<millimeter>>("diameter_y")
                .register()?,
            roundness: ctx.measurement::<Option<f64>>("roundness").register()?,
            subscription: None,
            subscribed_diameter,
            subscribed_diameter_x,
            subscribed_diameter_y,
            subscribed_roundness,
            last_request: Instant::now(),
        })
    }
}

impl Machine for LaserV1 {
    fn act(&mut self) -> ActResult {
        self.update_device()?;

        if let Some(m) = self.device.borrow().measurement.clone() {
            fn convert(value: u16) -> Length {
                Length::new::<millimeter>(value as f64 / 1000.0)
            }

            self.diameter.set(convert(m.diameter));
            self.diameter_x.set(Some(convert(m.x_axis)));
            self.diameter_y.set(Some(convert(m.y_axis)));
        }

        let roundness = self.compute_roundness();
        self.roundness.set(roundness);

        let in_tolerance = self.compute_in_tolerance();
        self.in_tolerance.set(in_tolerance);

        if let Some(subscription) = &mut self.subscription {
            // --- config ---
            self.subscribed_diameter_target
                .set(Some(subscription.diameter_target.get()))
                .unwrap();

            self.subscribed_diameter_tolerance_lower
                .set(Some(subscription.diameter_tolerance_lower.get()))
                .unwrap();

            self.subscribed_diameter_tolerance_upper
                .set(Some(subscription.diameter_tolerance_upper.get()))
                .unwrap();

            // --- state ---
            self.subscribed_in_tolerance
                .set(Some(subscription.in_tolerance.get()));

            // --- measurements ---
            self.subscribed_diameter
                .set(Some(subscription.diameter.get()));
            self.subscribed_diameter_x
                .set(subscription.diameter_x.get());
            self.subscribed_diameter_y
                .set(subscription.diameter_y.get());
            self.subscribed_roundness.set(subscription.roundness.get());
        }

        Ok(())
    }

    fn subscribe(&mut self, mut ctx: SubscribeContext) -> SubscribeResult {
        if ctx.provider().identification != LaserV1::IDENTIFICATION {
            return Err(SubscribeError::UnsupportedMachine);
        }

        if self.subscription.is_some() {
            return Err(SubscribeError::TooManySubscriptions);
        }

        let ident = ctx.provider();

        self.subscription = Some(LaserV1Subscription {
            ident,

            // --- config ---
            diameter_target: ctx.config("diameter.target")?,
            diameter_tolerance_lower: ctx.config("diameter.tolerance.lower")?,
            diameter_tolerance_upper: ctx.config("diameter.tolerance.upper")?,

            // --- state ---
            in_tolerance: ctx.state("in_tolerance")?,

            // --- measurements ---
            diameter: ctx.measurement("diameterz")?,
            diameter_x: ctx.measurement("diameter_x")?,
            diameter_y: ctx.measurement("diameter_y")?,
            roundness: ctx.measurement("roundness")?,
        });

        Ok(())
    }

    fn unsubscribe(&mut self, ident: MachineIdentificationUnique) {
        if let Some(subscription) = self.subscription.as_ref()
            && subscription.ident == ident
        {
            self.subscription = None;

            // --- config ---
            self.subscribed_diameter_target.set(None).unwrap();

            self.subscribed_diameter_tolerance_lower.set(None).unwrap();

            self.subscribed_diameter_tolerance_upper.set(None).unwrap();

            // --- state ---
            self.subscribed_in_tolerance.set(None);

            // --- measurements ---
            self.subscribed_diameter.set(None);
            self.subscribed_diameter_x.set(None);
            self.subscribed_diameter_y.set(None);
            self.subscribed_roundness.set(None);
        }
    }
}

impl LaserV1 {
    pub const IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor_id: 1,
        machine_id: 6,
    };

    fn update_device(&mut self) -> ActResult {
        let mut laser = self.device.borrow_mut();

        if let Err(e) = laser.handle_response()
            && let Some(laser_error) = e.downcast_ref::<LaserError>()
            && let LaserError::IoErr() = laser_error
        {
            return Err(ActError {
                recoverable: false,
                kind: ActErrorKind::HardwareFault("Physical hardware I/O broke.".into()),
            });
        }

        let now = Instant::now();
        if now.duration_since(self.last_request) > Duration::from_millis(6) {
            self.last_request = now;
            let res = laser.send_next_request();

            if res.is_err() {
                println!("send_next_request {:?}", res);
            }
        }

        Ok(())
    }

    /// Roundness = min(x, y) / max(x, y)
    fn compute_roundness(&mut self) -> Option<f64> {
        let (Some(x), Some(y)) = (
            self.diameter_x.get_as::<millimeter>(),
            self.diameter_y.get_as::<millimeter>(),
        ) else {
            return None;
        };

        if x > 0.0 && y > 0.0 {
            let roundness = f64::min(x, y) / f64::max(x, y);
            Some(roundness)
        } else if x == 0.0 && y == 0.0 {
            Some(0.0)
        } else {
            None
        }
    }

    /// Calculates if the current diameter is inside of the tolerance
    fn compute_in_tolerance(&mut self) -> bool {
        // const DIAMETER_EPSILON: f64 = 0.0001; // in mm
        //
        // // early return true if the diameter is 0 to prevent warning happening before start
        // if self.diameter.get_as::<millimeter>() < DIAMETER_EPSILON {
        //     return true;
        // }

        let target = self.diameter_target.get();
        let top = target + self.diameter_tolerance_upper.get();
        let bottom = target - self.diameter_tolerance_lower.get();

        self.diameter.get() < top && self.diameter.get() > bottom
    }
}
