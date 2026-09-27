use std::hash::Hash;

use crate::EntityId;

/// Base contract for domain entities.
///
/// Entity identity is separate from persistence concerns. Concrete bounded
/// contexts should define strongly typed ID wrappers such as UserId or
/// OrderId around EntityId.
pub trait Entity {
    type Id: Copy + Eq + Hash;

    fn id(&self) -> Self::Id;
}
