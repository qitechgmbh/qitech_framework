// ctx.modbus_holding_register::<millimeter>(&slot, 1)
// ctx.modbus_input_register::<millimeter>(&slot, 1)
// ctx.modbus.state_property::<>(),

// ctx.modbus_input_register(&slot, 1)

// ReadonlyProperty
//

/*
let laser_v1_0 = rt.machine(1, LaserV1::builder());
let laser_v1_1 = rt_other.machine(1, LaserV1::builder());
let laser_v1_2 = LaserV1::builder();

rt.modbus_rtu(
    ModbusRtuBusConfig::new(
        ModbusRtuPort::topology("pci-0000:c6:00.0-usbv2-0:2:1.0-port0"),
        19200,
    ).parity(Parity::None)
)
    .bind(&mut laser_v1_0.device, 1)?
    .bind(&mut laser_v1_1.device, 2)?
    .bind(&mut laser_v1_2.device, 3)?;
*/
