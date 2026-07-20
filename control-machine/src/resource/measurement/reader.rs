// --- reader ---
#[derive(Debug)]
pub struct MeasurementReaderHandle<const REGISTRY_ID: usize, T> {
    generation: u64,
    index: usize,
    _marker: PhantomData<T>,
}

pub struct MeasurementReader<'a, const REGISTRY_ID: usize, const MAX_ITEMS: usize> {
    registry: &'a MeasurementRegistry<REGISTRY_ID, MAX_ITEMS>,
}

impl<'a, const REGISTRY_ID: usize, const MAX_ITEMS: usize> 
    MeasurementReader<'a, REGISTRY_ID, MAX_ITEMS>
{
    pub(crate) fn new(registry: &'a MeasurementRegistry<REGISTRY_ID, MAX_ITEMS>) -> Self {
        Self { registry }
    }

    pub fn read<T: WrappedTryFromOptionalF64>(
        &self,
        handle: &MeasurementReaderHandle<REGISTRY_ID, T>,
    ) -> Result<T::Inner, ReadError> {
        let generation = self.registry.buf_generations[handle.index];

        // Safety:
        // - register() verified T fits in storage
        // - register() stored TypeId::of::<T>()
        // - resolve() only creates PropertyHandle<T> after type check
        unsafe {
            if generation.assume_init() != handle.generation {
                return Err(ReadError);
            }

            let null = self.registry.buf_nulls[handle.index].assume_init_read();

            let value = if !null {
                Some(self.registry.buf_values[handle.index].assume_init_read())
            } else { None };
            
            let value = T::try_from_opt_f64(value).expect("T not allow to be None, found None!");
            Ok(value)
        }
    }
}

// --- resolver ---
pub struct MeasurementResolver<'a, const REGISTRY_ID: usize, const MAX_ITEMS: usize> {
    registry: &'a MeasurementRegistry<REGISTRY_ID, MAX_ITEMS>,
    ident: MachineIdentificationUnique,
}

impl<'a, const REGISTRY_ID: usize, const MAX_ITEMS: usize> MeasurementResolver<'a, REGISTRY_ID, MAX_ITEMS> {
    pub fn resolve<T: 'static>(
        &self,
        name: &'static str,
    ) -> Result<MeasurementReaderHandle<REGISTRY_ID, T>, ResolveError> {
        let key = Key { ident: self.ident, name };

        let Some(Entry { index, type_id }) = self.registry.lookup.get(&key) else {
            return Err(ResolveError::NoSuchProperty)
        };
        
        if *type_id != TypeId::of::<T>() {
            return Err(ResolveError::InvalidType);
        }

        let generation = unsafe {
            self.registry.buf_generations[*index].assume_init()
        };

        Ok(MeasurementReaderHandle {
            generation,
            index: *index,
            _marker: PhantomData,
        })
    }
}
