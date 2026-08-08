use std::any::TypeId;
use std::any::type_name;
use std::ptr::NonNull;

use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::ResourceAccessError;
use qitech_framework_core::report::ResourceKind;

use crate::machine::BumpAllocator;
use crate::machine::BumpAllocatorMark;

pub struct PropertyRegistry {
    kind: ResourceKind,
    descriptors: Vec<SlotDescriptor>,
    alloc_value: BumpAllocator,
    alloc_cache: BumpAllocator,
}

impl PropertyRegistry {
    pub fn new(kind: ResourceKind, pool_size: usize) -> Self {
        Self {
            kind,
            descriptors: Vec::new(),
            alloc_value: BumpAllocator::new(pool_size),
            alloc_cache: BumpAllocator::new(pool_size),
        }
    }

    pub fn register(&mut self) -> PropertyRegistrar<'_> {
        PropertyRegistrar {
            descriptors_pos: self.descriptors.len(),
            mark_value: self.alloc_value.mark(),
            mark_cache: self.alloc_cache.mark(),
            registry: self,
            committed: false,
        }
    }

    pub fn get_cached<T: 'static>(
        &mut self,
        ident: MachineIdentificationUnique,
        resource: &'static str,
    ) -> Result<NonNull<T>, ResourceAccessError> {
        let type_id = TypeId::of::<T>();
        let type_name = type_name::<T>();

        let mut machine_found = false;

        for descriptor in &self.descriptors {
            if descriptor.ident != ident {
                continue;
            }

            machine_found = true;

            if descriptor.resource != resource {
                continue;
            }

            if descriptor.type_id != type_id {
                return Err(ResourceAccessError::TypeMismatch {
                    actual: descriptor.type_name.to_owned(),
                    received: type_name.to_owned(),
                });
            }

            return Ok(descriptor.p_cache.cast());
        }

        if machine_found {
            Err(ResourceAccessError::ResourceNotFound {
                kind: self.kind,
                path: resource.to_string(),
            })
        } else {
            Err(ResourceAccessError::MachineNotFound)
        }
    }

    pub fn sync_cache(&mut self) {
        // --- copy snapshot of values into cache ---
        self.alloc_cache.sync(&self.alloc_value);
    }
}

#[derive(Debug, Clone, Copy)]
struct SlotDescriptor {
    ident: MachineIdentificationUnique,
    resource: &'static str,
    type_id: TypeId,
    type_name: &'static str,

    p_value: NonNull<()>,
    p_cache: NonNull<()>,
}

// --- registrar ---
pub struct PropertyRegistrar<'a> {
    registry: &'a mut PropertyRegistry,
    descriptors_pos: usize,
    mark_value: BumpAllocatorMark,
    mark_cache: BumpAllocatorMark,
    committed: bool,
}

impl PropertyRegistrar<'_> {
    pub fn register<T: Clone + 'static>(
        &mut self,
        ident: MachineIdentificationUnique,
        resource: &'static str,
        value: T,
    ) -> NonNull<T> {
        let type_id = TypeId::of::<T>();
        let type_name = type_name::<T>();

        let registered_slot = self
            .registry
            .descriptors
            .iter()
            .position(|slot| slot.ident == ident && slot.resource == resource);

        if let Some(slot) = registered_slot {
            let descriptor = &self.registry.descriptors[slot];

            assert_eq!(
                descriptor.type_id, type_id,
                "property type mismatch for {resource}: \
                 already registered as {}, attempted to register as {type_name}",
                descriptor.type_name,
            );

            unsafe {
                descriptor.p_value.cast().write(value);
            };
            return descriptor.p_value.cast();
        }

        let p_value = self.registry.alloc_value.allocate::<T>();
        let p_cache = self.registry.alloc_cache.allocate::<T>();

        unsafe {
            // --- initialize memory with value ---
            p_value.cast().write(value.clone());
            p_cache.cast().write(value.clone());
        }

        self.registry.descriptors.push(SlotDescriptor {
            ident,
            resource,
            type_id,
            type_name,
            p_value: p_value.cast(),
            p_cache: p_cache.cast(),
        });

        p_value
    }

    pub fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for PropertyRegistrar<'_> {
    fn drop(&mut self) {
        if self.committed {
            return;
        }

        self.registry.alloc_value.rollback(self.mark_value);

        self.registry.alloc_cache.rollback(self.mark_cache);

        self.registry.descriptors.truncate(self.descriptors_pos);
    }
}
