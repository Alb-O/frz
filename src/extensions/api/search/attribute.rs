/// Represents a single attribute row with its label and the number of matches.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttributeRow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    pub name: String,
    pub count: usize,
}

impl AttributeRow {
    /// Create a new [`AttributeRow`] with the provided `name` and `count`.
    #[must_use]
    pub fn new(name: impl Into<String>, count: usize) -> Self {
        let name = name.into();
        let id = Some(super::identity::stable_hash64(&name));
        Self { name, count, id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_row_uses_provided_values() {
        let row = AttributeRow::new("tag", 3);
        assert!(row.id.is_some());
        assert_eq!(row.name, "tag");
        assert_eq!(row.count, 3);
    }
}
