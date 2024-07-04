pub enum ResourceType {
    Coal,
    Iron,
    Steel,
}

impl ResourceType {
    #[must_use]
    pub const fn all() -> [Self; 3] {
        [ResourceType::Coal, ResourceType::Iron, ResourceType::Steel]
    }
}
