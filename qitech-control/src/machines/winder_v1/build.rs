use std::{cell::RefCell, rc::Rc};
use qitech_lib::ethercat_hal::{EtherCATThreadChannel, shared_config};
use qitech_lib::ethercat_hal::coe::ConfigurableDevice;
use qitech_lib::ethercat_hal::shared_config::el70x1::{EL70x1OperationMode, EL70x1SpeedRange, StmMotorConfiguration};
use qitech_lib::ethercat_hal::devices::{
    ek1100::EK1100, 
    el7031::{EL7031, coe::EL7031Configuration, pdo::EL7031PredefinedPdoAssignment}, 
    el7031_0030::{self, EL7031_0030, coe::EL7031_0030Configuration, pdo::EL7031_0030PredefinedPdoAssignment}, 
    el7041_0052::{EL7041_0052, coe::EL7041_0052Configuration},
};

use control_runtime::{MachineBuild, MachineBuildError, BuildContext};
use crate::machines::winder_v1::tension_arm::TensionArm;

use super::WinderV1;

impl MachineBuild for WinderV1 {
    fn build(
        mut builder: BuildContext
    ) -> Result<Self, MachineBuildError> {
        // --- init hardware ---
        _ = builder.get_ethercat_device::<EK1100>(0)?;
        let interface   = builder.get_ethercat_interface()?;
        let el7041_0052 = init_el7041_0052(&mut builder, &interface)?;
        let el7031      = init_el7031(&mut builder, &interface)?;
        let el7031_0030 = init_el7031_0030(&mut builder, &interface)?;

        _ = el7041_0052;
        _ = el7031;

        // --- virtual devices ---
        let tension_arm = TensionArm::new(
            el7031_0030.clone(), 
            builder.state("tension_arm.zero").register(), 
            builder.measurement("tension_arm.angle").register()
        );

        Ok(Self {
            mode: builder.state("mode").register()?,
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
    builder: &mut BuildContext,
    interface: &EtherCATThreadChannel,
) -> anyhow::Result<Rc<RefCell<EL7041_0052>>> {
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

    let (dev, addr) = builder.find_ethercat_device_and_addr::<EL7041_0052>(2)?;
    dev.borrow_mut().write_config(interface.clone(), addr, &config)?;
    interface.enable_dc_sync0(addr)?;
    Ok(dev)
}

/// Role 3: Stepper Traverse EL7031
fn init_el7031(
    builder: &mut BuildContext,
    interface: &EtherCATThreadChannel,
) -> anyhow::Result<Rc<RefCell<EL7031>>> {
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

    let (dev, addr) = builder.find_ethercat_device_and_addr::<EL7031>(3)?;
    dev.borrow_mut().write_config(interface.clone(), addr, &config)?;
    interface.enable_dc_sync0(addr)?;
    Ok(dev)
}

/// Role 4: Stepper Puller EL7031-0030
fn init_el7031_0030(
    builder: &mut BuildContext,
    interface: &EtherCATThreadChannel,
) -> anyhow::Result<Rc<RefCell<EL7031_0030>>> {
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

    let (dev, addr) = builder.find_ethercat_device_and_addr::<EL7031_0030>(4)?;
    dev.borrow_mut().write_config(interface.clone(), addr, &config)?;
    interface.enable_dc_sync0(addr)?;
    Ok(dev)
}
