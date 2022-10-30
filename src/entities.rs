use std::fmt::Debug;

use downcast_rs::{impl_downcast, Downcast};
use glam::{vec3, Vec3};
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};
use thiserror::Error;
use uuid::Uuid;

fn main() {
    let entity = Entity::new().with_physics(Some(PhysicsObject {
        velocity: Vec3::ZERO,
    }));

    let mut entities = vec![entity];

    gravity_system(&mut entities, 1.0);
}

pub trait HasID {
    fn id(&self) -> Uuid;
}

#[derive(Default, Debug)]
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
    #[error("event needs physicsobject but it is None")]
    NoPhysics,
}

pub trait Component: Downcast + Debug {}

impl_downcast!(Component);

#[derive(Debug, Default)]
struct PhysicsObject {
    pub velocity: Vec3,
}
impl Component for PhysicsObject {}

#[derive(Debug, Default)]
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
pub enum EntityEvent {
    PositionChange { to: Vec3 },
    VelocityChange { to: Vec3 },
}

fn gravity_system(entities: &mut [Entity], delta: f32) -> Vec<Event> {
    entities
        .par_iter_mut()
        .filter_map(|v| {
            if let Some(physics) = &mut v.physics {
                Some((physics, v.id))
            } else {
                None
            }
        })
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
