use std::{collections::HashMap, fmt::Debug};

use downcast_rs::{impl_downcast, Downcast};
use glam::{vec3, Vec3};
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};
use thiserror::Error;
use uuid::Uuid;

fn main() {
    let entity = Entity::new().with_physics(Some(PhysicsObject {
        velocity: Vec3::ZERO,
    }));
    let entity_id = entity.id;

    let mut world = World::new();
    world.add_entity(entity);

    let mut events = Vec::new();

    println!("before: {:?}", world.entities.get(&entity_id).unwrap());
    for tick in 0..=1 {
        events.append(&mut gravity_system(&mut world.entities, 1.0));
        events.append(&mut physics_system(&mut world.entities, 1.0));
        world.apply_events(&mut events).unwrap();

        println!(
            "tick {}:  {:?}",
            tick,
            world.entities.get(&entity_id).unwrap()
        );
    }
}

struct World {
    pub entities: HashMap<Uuid, Entity>,
}
impl World {
    fn new() -> Self {
        Self {
            entities: HashMap::new(),
        }
    }
    fn add_entity(&mut self, entity: Entity) {
        self.entities.insert(entity.id, entity);
    }
    fn apply_events(&mut self, events: &mut Vec<Event>) -> Result<(), EventApplyError> {
        for event in events.drain(..) {
            match event {
                Event::Entity(id, event) => self
                    .entities
                    .get_mut(&id)
                    .ok_or(EventApplyError::EntityNotFound)?
                    .apply_event(event)?,
            }
        }

        Ok(())
    }
}

pub trait HasID {
    fn id(&self) -> Uuid;
}

#[derive(Default, Debug, Clone)]
struct Entity {
    id: Uuid,
    transform: Transform,
    physics: Option<PhysicsObject>,
}
impl HasID for Entity {
    fn id(&self) -> Uuid {
        self.id
    }
}
impl Entity {
    fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            transform: Transform {
                position: Vec3::ZERO,
            },
            physics: None,
        }
    }
    fn with_physics(mut self, physics: Option<PhysicsObject>) -> Self {
        self.physics = physics;
        self
    }

    fn apply_event(&mut self, event: EntityEvent) -> Result<(), EventApplyError> {
        match event {
            EntityEvent::PositionChange { to } => self.transform.position = to,
            EntityEvent::VelocityChange { to } => {
                let physics = self.physics.as_mut().ok_or(EventApplyError::NoPhysics)?;
                physics.velocity = to;
            }
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum EventApplyError {
    #[error("an entity was not found")]
    EntityNotFound,

    #[error("event needs physicsobject but it is None")]
    NoPhysics,
}

pub trait Component: Downcast + Debug {}

impl_downcast!(Component);

#[derive(Debug, Default, Clone)]
struct PhysicsObject {
    pub velocity: Vec3,
}
impl Component for PhysicsObject {}

#[derive(Debug, Default, Clone)]
struct Transform {
    pub position: Vec3,
}
impl Component for Transform {}

pub enum Event {
    Entity(Uuid, EntityEvent),
}

impl Event {
    pub fn entity(entity: Uuid, event: EntityEvent) -> Self {
        Self::Entity(entity, event)
    }
}

#[derive(Clone, Debug)]
pub enum EntityEvent {
    PositionChange { to: Vec3 },
    VelocityChange { to: Vec3 },
}

fn gravity_system(entities: &mut HashMap<Uuid, Entity>, delta: f32) -> Vec<Event> {
    entities
        .par_iter_mut()
        .filter_map(|(id, v)| v.physics.as_mut().map(|physics| (physics, *id)))
        .map(|(physics, id)| {
            Event::entity(
                id,
                EntityEvent::VelocityChange {
                    to: physics.velocity + vec3(0., -9.81 * delta, 0.),
                },
            )
        })
        .collect()
}

fn physics_system(entities: &mut HashMap<Uuid, Entity>, delta: f32) -> Vec<Event> {
    entities
        .par_iter_mut()
        .filter_map(|(id, v)| {
            v.physics
                .as_mut()
                .map(|physics| (physics, &mut v.transform, *id))
        })
        .map(|(physics, transform, id)| {
            Event::entity(
                id,
                EntityEvent::PositionChange {
                    to: transform.position + (physics.velocity * delta),
                },
            )
        })
        .collect()
}
