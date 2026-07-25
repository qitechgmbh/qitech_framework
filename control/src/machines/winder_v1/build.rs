use std::cell::RefCell;
use std::rc::Rc;

use qitech_framework::machine::BuildContext;
use qitech_framework::machine::MachineBuild;
use qitech_framework::machine::error::BuildResult;
use qitech_framework::machine::resource::MeasurementRegisterOptions;
use qitech_framework::uom::Angle;
use qitech_framework::uom::ConstZero;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use qitech_lib::ethercat_hal::coe::ConfigurableDevice;
use qitech_lib::ethercat_hal::devices::ek1100::EK1100;
use qitech_lib::ethercat_hal::devices::el7031::EL7031;
use qitech_lib::ethercat_hal::devices::el7031::coe::EL7031Configuration;
use qitech_lib::ethercat_hal::devices::el7031::pdo::EL7031PredefinedPdoAssignment;
use qitech_lib::ethercat_hal::devices::el7031_0030::EL7031_0030;
use qitech_lib::ethercat_hal::devices::el7031_0030::coe::EL7031_0030Configuration;
use qitech_lib::ethercat_hal::devices::el7031_0030::pdo::EL7031_0030PredefinedPdoAssignment;
use qitech_lib::ethercat_hal::devices::el7031_0030::{self};
use qitech_lib::ethercat_hal::devices::el7041_0052::EL7041_0052;
use qitech_lib::ethercat_hal::devices::el7041_0052::coe::EL7041_0052Configuration;
use qitech_lib::ethercat_hal::shared_config;
use qitech_lib::ethercat_hal::shared_config::el70x1::EL70x1OperationMode;
use qitech_lib::ethercat_hal::shared_config::el70x1::EL70x1SpeedRange;
use qitech_lib::ethercat_hal::shared_config::el70x1::StmMotorConfiguration;

use super::WinderV1;
use crate::machines::winder_v1::tension_arm::TensionArm;

impl MachineBuild for WinderV1 {
    fn build(mut ctx: BuildContext) -> BuildResult<Self> {
        // --- init hardware ---
        _ = ctx.get_ethercat_device::<EK1100>(0)?;
        let interface = ctx.get_ethercat_interface()?;
        let el7041_0052 = init_el7041_0052(&mut ctx, &interface)?;
        let el7031 = init_el7031(&mut ctx, &interface)?;
        let el7031_0030 = init_el7031_0030(&mut ctx, &interface)?;

        _ = el7041_0052;
        _ = el7031;

        // --- virtual devices ---
        let tension_arm = TensionArm::new(
            el7031_0030.clone(),
            ctx.register_state("tension_arm.zero", None)?,
            ctx.register_measurement(
                "tension_arm.angle",
                MeasurementRegisterOptions {
                    initial: Some(Angle::ZERO),
                    ..Default::default()
                },
            )?,
        );

        Ok(Self {
            mode: ctx.register_state("mode")?,
            tension_arm,
            commands: todo!(),
            spool: todo!(),
            puller: todo!(),
            travserse: todo!(),
            spool_target: todo!(),
            laser_subscription: todo!(),
        })
    }
}

/// Role no.2
fn init_el7041_0052(
    ctx: &mut BuildContext,
    interface: &EtherCATThreadChannel,
) -> BuildResult<Rc<RefCell<EL7041_0052>>> {
    let config = EL7041_0052Configuration {
        stm_features: shared_config::el70x1::StmFeatures {
            operation_mode: EL70x1OperationMode::DirectVelocity,
            ..Default::default()
        },
        stm_motor: StmMotorConfiguration {
            max_current: 2800,
            ..Default::default()
        },
        ..Default::default()
    };

    let (dev, addr) = ctx.find_ethercat_device_and_addr::<EL7041_0052>(2)?;
    dev.borrow_mut()
        .write_config(interface.clone(), addr, &config)?;
    interface.enable_dc_sync0(addr)?;
    Ok(dev)
}

/// Role 3: Stepper Traverse EL7031
fn init_el7031(
    ctx: &mut BuildContext,
    interface: &EtherCATThreadChannel,
) -> BuildResult<Rc<RefCell<EL7031>>> {
    let config = EL7031Configuration {
        stm_features: shared_config::el70x1::StmFeatures {
            operation_mode: EL70x1OperationMode::DirectVelocity,
            speed_range: shared_config::el70x1::EL70x1SpeedRange::Steps1000,
            ..Default::default()
        },
        stm_motor: StmMotorConfiguration {
            max_current: 1500,
            ..Default::default()
        },
        pdo_assignment: EL7031PredefinedPdoAssignment::VelocityControlCompact,
        ..Default::default()
    };

    let (dev, addr) = ctx.find_ethercat_device_and_addr::<EL7031>(3)?;
    dev.borrow_mut()
        .write_config(interface.clone(), addr, &config)?;
    interface.enable_dc_sync0(addr)?;
    Ok(dev)
}

/// Role 4: Stepper Puller EL7031-0030
fn init_el7031_0030(
    ctx: &mut BuildContext,
    interface: &EtherCATThreadChannel,
) -> BuildResult<Rc<RefCell<EL7031_0030>>> {
    use el7031_0030::coe::StmFeatures;

    let config = EL7031_0030Configuration {
        stm_features: StmFeatures {
            operation_mode: EL70x1OperationMode::DirectVelocity,
            speed_range: EL70x1SpeedRange::Steps1000,
            ..Default::default()
        },
        stm_motor: StmMotorConfiguration {
            max_current: 2700,
            ..Default::default()
        },
        pdo_assignment: EL7031_0030PredefinedPdoAssignment::VelocityControlCompact,
        ..Default::default()
    };

    let (dev, addr) = ctx.find_ethercat_device_and_addr::<EL7031_0030>(4)?;
    dev.borrow_mut()
        .write_config(interface.clone(), addr, &config)?;
    interface.enable_dc_sync0(addr)?;
    Ok(dev)
}
