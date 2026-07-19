impl<'a> MachineBuilder<'a> {
    pub fn measurement<'b, T, U>(
        &'b mut self,
        name: &'static str,
    ) -> MeasurementBuilder<'a, 'b, T, U>
    where
        'a: 'b,
        T: Copy + Default,
    {
        MeasurementBuilder {
            root: self,
            name,
            record_min: false,
            record_max: false,
            initial_value: Default::default(),
            _marker: PhantomData,
        }
    }
}

pub struct MeasurementBuilder<'a, 'b, T, U>
where
    T: Copy + Default,
{
    root: &'b mut MachineBuilder<'a>,
    name: &'static str,
    record_min: bool,
    record_max: bool,
    initial_value: T,
    _marker: PhantomData<U>,
}

impl<T: Copy + Default, N> MeasurementBuilder<'_, '_, T, N>
where
    T: Copy + Default,
{
    pub fn record_min(&mut self) -> &mut Self {
        self.record_min = true;
        self
    }

    pub fn record_max(&mut self) -> &mut Self {
        self.record_max = true;
        self
    }

    pub fn initial_value(&mut self, value: T) -> &mut Self {
        self.initial_value = value;
        self
    }

    pub fn register(self) -> Measurement<T, N> {
        let ident = self.root.ident;

        let reg = &mut self.root.data_store.registry;
        let name = reg.register_name(self.name.to_string());
        let handle = reg.register_measurement(ident, name, false).unwrap();

        let min = if self.record_min {
            let name = reg.register_name(format!("{name}.min"));
            let handle = reg.register_measurement(ident, name, true).unwrap();
            Some(handle)
        } else {
            None
        };

        let max = if self.record_max {
            let name = reg.register_name(format!("{name}.max"));
            let handle = reg.register_measurement(ident, name, true).unwrap();
            Some(handle)
        } else {
            None
        };

        let stats = MeasurementStatistics::new(min, max);
        Measurement::new(handle, stats, self.initial_value)
    }
}
