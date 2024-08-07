use crate::world::{unsafe_world_cell::UnsafeWorldCell, World};

use super::{BoxedSystem, BuildableSystemParam, IntoSystem, System, SystemParam};

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

    #[inline]
    pub fn inner_system(&mut self) -> &mut BoxedSystem<In, Out> {
        &mut self.state.system
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

pub struct InnerSystemBuilder<'b, In = (), Out = ()> {
    world: &'b mut World,
    system: Option<BoxedSystem<In, Out>>,
}

impl<'b, In, Out> InnerSystemBuilder<'b, In, Out> {
    #[inline]
    pub fn new(world: &'b mut World) -> Self {
        Self {
            world,
            system: None,
        }
    }

    #[inline]
    pub fn world(&mut self) -> &mut World {
        self.world
    }

    #[inline]
    pub fn with_system<Marker>(&mut self, system: impl IntoSystem<In, Out, Marker>)
    where
        Marker: 'static,
    {
        assert!(
            self.system.is_none(),
            "`InnerSystemBuilder` was initialized with a system twice"
        );
        let system = Box::new(IntoSystem::into_system(system));
        assert_eq!(
            Some(self.world.id()),
            system.world_id(),
            "TODO error" // TODO better error
        );
        self.system = Some(system);
    }
}

impl<'w, 's, In, Out> BuildableSystemParam for InnerSystem<'w, 's, In, Out>
where
    In: 'static,
    Out: 'static,
{
    type Builder<'b> = InnerSystemBuilder<'b, In, Out>;

    fn build(
        world: &mut crate::prelude::World,
        meta: &mut super::SystemMeta,
        func: impl FnOnce(&mut Self::Builder<'_>),
    ) -> Self::State {
        let mut builder = InnerSystemBuilder::new(world);
        func(&mut builder);
        let Some(system) = builder.system.take() else {
            panic!("`InnerSystemBuilder` was not given a system to initialize. Use `with_system`.")
        };
        assert!(
            meta.component_access_set
                .is_compatible(system.component_access_set()),
            "TODO error here" // TODO
        );
        InnerSystemState { system }
    }
}
