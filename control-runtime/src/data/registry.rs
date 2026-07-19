use control_core::MachineIdentificationUnique;

const NAMES_COUNT_MAX: usize = 2048;
const NAME_LEN_MAX: usize = 96;

use super::measurement;

#[derive(Debug)]
pub struct DataRegistry {
    names: heapless::Vec<&'static str, NAMES_COUNT_MAX>,
    // config: PropertyRegistry<256>,
    // state: PropertyRegistry<256>,
    pub measurement: measurement::Registry<0, 512>,
}

impl DataRegistry {
    pub(crate) fn new() -> Self {
        Self {
            names: Default::default(),
            // config: PropertyRegistry::new(),
            // state: PropertyRegistry::new(),
            measurement: measurement::Registry::new(),
        }
    }

    /// Interns a name: returns the existing `&'static str` if already
    /// registered, otherwise leaks and registers a new one. Bounded by the
    /// vec limit, so worst case is ~0.2 MiB (2048 * 96). 
    /// Avoids reallocating on every clone without multi-threading issues.
    pub(crate) fn register_name(&mut self, name: String) -> &'static str {
        // ensure name is not unreasonably large
        assert!(name.len() <= NAME_LEN_MAX);

        if let Some(&existing) = self.names.iter().find(|x| **x == name) {
            // entry exists. return
            return existing;
        }

        let leaked: &'static str = name.leak();
        self.names.push(leaked).expect("Not supposed to overflow .. EVER");
        leaked
    }

    // pub(crate) fn register_config(
    //     &mut self,
    //     ident: MachineIdentificationUnique,
    //     name: &'static str,
    // ) -> Result<PropertyHandle, String> {
    //     Self::register_property(&mut self.config, ident, name)
    // }

    // pub(crate) fn register_state(
    //     &mut self,
    //     ident: MachineIdentificationUnique,
    //     name: &'static str,
    // ) -> Result<PropertyHandle, String> {
    //     Self::register_property(&mut self.state, ident, name)
    // }

    // pub(crate) fn register_measurement(
    //     &mut self,
    //     ident: MachineIdentificationUnique,
    //     name: &'static str,
    //     reset_on_clear: bool,
    // ) -> Result<MeasurementHandle, String> {
    //     let idx = Self::get_idx(&mut self.measurement.active_list)?;
// 
    //     let reg = &mut self.measurement;
    //     reg.idents_buf[idx] = ident;
    //     reg.names_buf[idx] = name;
// 
    //     if reset_on_clear {
    //         reg.reset_list.push(idx).expect("Must not be full");
    //     }
// 
    //     unsafe {
    //         let p_value = &mut reg.values_buf[idx] as *mut f64;
    //         let p_value = NonNull::new_unchecked(p_value);
// 
    //         let p_null = &mut reg.nulls_buf[idx] as *mut bool;
    //         let p_null = NonNull::new_unchecked(p_null);
// 
    //         Ok(MeasurementHandle {
    //             p_value,
    //             p_is_null: p_null,
    //         })
    //     }
    // }

    // fn register_property<const N: usize>(
    //     registry: &mut PropertyRegistry<N>,
    //     ident: MachineIdentificationUnique,
    //     name: &'static str,
    // ) -> Result<PropertyHandle, String> {
    //     let idx = Self::get_idx(&mut registry.active_list)?;
    //     registry.idents_buf[idx] = ident;
    //     registry.names_buf[idx] = name;
    //     let p_value = &mut registry.values_buf[idx] as *mut ScalarValue;
    //     let p_value = unsafe { NonNull::new_unchecked(p_value) };
    //     Ok(PropertyHandle { p_value })
    // }

    fn get_idx<const N: usize>(active_list: &mut heapless::Vec<bool, N>) -> Result<usize, String> {
        match active_list.iter().position(|active| !*active) {
            Some(idx) => Ok(idx),
            None => {
                let idx = active_list.len();
                active_list.push(true).map_err(|_| "full".to_string())?;
                Ok(idx)
            }
        }
    }

    /// TODO: REFLECT CHANGES IN CLEAR_LISTS IMPORTANT
    pub fn unregister_machine(&mut self, ident: MachineIdentificationUnique) {
        for (idx, active) in &mut self.config.active_list.iter_mut().enumerate() {
            if !*active { continue; }

            if self.config.idents_buf[idx] == ident {
                *active = false;
            }
        }

        for (idx, active) in &mut self.state.active_list.iter_mut().enumerate() {
            if !*active { continue; }

            if self.state.idents_buf[idx] == ident {
                *active = false;
            }
        }

        for (idx, active) in &mut self.measurement.active_list.iter_mut().enumerate() {
            if !*active { continue; }

            if self.measurement.idents_buf[idx] == ident {
                *active = false;
            }
        }
    }
}
