use uuid::Uuid;

/// Base UUIDv7-backed identity value for domain-specific ID newtypes.
///
/// Bounded contexts should wrap this type in a concrete newtype rather than
/// using it directly, for example: pub struct UserId(EntityId).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(Uuid);

impl EntityId {
    /// Generate a new UUIDv7 identity using the current system time.
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Restore an identity from a UUID value.
    ///
    /// Only UUIDv7 values are accepted so the identity strategy remains
    /// explicit at the domain boundary.
    pub fn from_uuid(value: Uuid) -> Option<Self> {
        (value.get_version_num() == 7).then_some(Self(value))
    }

    /// Return the underlying UUID for persistence or integration adapters.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Consume the identity and return its UUID representation.
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Version;

    #[test]
    fn new_generates_uuid_v7() {
        let id = EntityId::new();

        assert_eq!(id.as_uuid().get_version(), Some(Version::SortRand));
        assert_eq!(id.as_uuid().get_version_num(), 7);
    }

    #[test]
    fn from_uuid_accepts_uuid_v7() {
        let uuid = Uuid::now_v7();

        assert!(EntityId::from_uuid(uuid).is_some());
    }

    #[test]
    fn from_uuid_rejects_non_v7_uuid() {
        let uuid = Uuid::nil();

        assert!(EntityId::from_uuid(uuid).is_none());
    }

    #[test]
    fn ids_are_ordered_by_generation_within_the_same_process() {
        let first = EntityId::new();
        let second = EntityId::new();

        assert!(first.as_uuid() < second.as_uuid());
    }
}
