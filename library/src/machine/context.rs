pub struct ReactContext<'a> {
    pub config: ConfigPropertyReader<'a>,
    pub state: StatePropertyReader<'a>,
    pub measurements: MeasurementReader<'a>,
}

pub struct SubscribeContext<'a> {
    pub ident: MachineIdentificationUnique,
    pub config: ConfigPropertyResolver<'a>,
    pub state: StatePropertyResolver<'a>,
    pub measurements: MeasurementResolver<'a>,
}

// command validation: builder creates list of path to variable + the validation to apply to it