use crate::Entity;

/// Marker contract for aggregate roots.
///
/// Aggregate roots are the only entities that external application code
/// should address directly across an aggregate boundary.
pub trait AggregateRoot: Entity {}
