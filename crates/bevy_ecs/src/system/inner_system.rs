use crate::world::{unsafe_world_cell::UnsafeWorldCell, World};

use super::{BoxedSystem, System, SystemParam, SystemParamBuilder};

pub struct InnerSystemState<In = (), Out = ()> {
    system: BoxedSystem<In, Out>,
}

pub struct InnerSystem<'w, 's, In = (), Out = ()> {
    world: UnsafeWorldCell<'w>,
    state: &'s mut InnerSystemState<In, Out>,
}

impl<'w, 's, In, Out> InnerSystem<'w, 's, In, Out>
where
    In: 'static,
    Out: 'static,
{
    #[inline]
    pub fn run(&mut self, input: In) -> Out {
        unsafe { self.state.system.run_unsafe(input, self.world) }
    }
}

unsafe impl<'w, 's, In, Out> SystemParam for InnerSystem<'w, 's, In, Out>
where
    In: 'static,
    Out: 'static,
{
    type State = InnerSystemState<In, Out>;

    type Item<'world, 'state> = InnerSystem<'world, 'state, In, Out>;

    fn init_state(
        world: &mut crate::prelude::World,
        system_meta: &mut super::SystemMeta,
    ) -> Self::State {
        todo!("Must be constructed through builder") // TODO better error
    }

    unsafe fn get_param<'world, 'state>(
        state: &'state mut Self::State,
        _system_meta: &super::SystemMeta,
        world: crate::world::unsafe_world_cell::UnsafeWorldCell<'world>,
        _change_tick: crate::component::Tick,
    ) -> Self::Item<'world, 'state> {
        InnerSystem { world, state }
    }

    fn apply(
        state: &mut Self::State,
        _system_meta: &super::SystemMeta,
        world: &mut crate::prelude::World,
    ) {
        state.system.apply_deferred(world);
    }

    unsafe fn new_archetype(
        state: &mut Self::State,
        archetype: &crate::archetype::Archetype,
        system_meta: &mut super::SystemMeta,
    ) {
        state.system.new_archetype(archetype);
        system_meta
            .archetype_component_access
            .extend(state.system.archetype_component_access());
    }

    fn queue(
        state: &mut Self::State,
        _system_meta: &super::SystemMeta,
        world: crate::world::DeferredWorld,
    ) {
        state.system.queue_deferred(world);
    }
}

pub struct InnerSystemBuilder<In = (), Out = ()> {
    system: BoxedSystem<In, Out>,
}

impl<In, Out> InnerSystemBuilder<In, Out> {
    pub fn new(system: impl System<In = In, Out = Out>) -> Self {
        Self {
            system: Box::new(system),
        }
    }
}

unsafe impl<'w, 's, 'b, In, Out> SystemParamBuilder<InnerSystem<'w, 's, In, Out>>
    for InnerSystemBuilder<In, Out>
where
    In: 'static,
    Out: 'static,
{
    fn build(self, world: &mut World, meta: &mut super::SystemMeta) -> InnerSystemState<In, Out> {
        let system = self.system;
        assert_eq!(system.world_id().unwrap(), world.id());
        let component_access = system.component_access_set();
        let archetype_component_access = system.archetype_component_access();
        assert!(component_access.is_compatible(&meta.component_access_set));
        assert!(archetype_component_access.is_compatible(&archetype_component_access)); // TODO not sure if needed -> component access check should be enough!
        meta.component_access_set.extend(component_access);
        meta.archetype_component_access
            .extend(archetype_component_access);
        InnerSystemState { system }
    }
}
