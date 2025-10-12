/// Represents a single facet row with its label and the number of matches.
#[derive(Debug, Clone)]
pub struct FacetRow {
    pub name: String,
    pub count: usize,
}

impl FacetRow {
    /// Create a new [`FacetRow`] with the provided `name` and `count`.
    #[must_use]
    pub fn new(name: impl Into<String>, count: usize) -> Self {
        Self {
            name: name.into(),
            count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_row_uses_provided_values() {
        let row = FacetRow::new("tag", 3);
        assert_eq!(row.name, "tag");
        assert_eq!(row.count, 3);
    }
}
