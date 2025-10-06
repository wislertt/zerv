use indexmap::IndexMap;
use serde::{
    Deserialize,
    Serialize,
};

/// Precedence levels for version components and schema sections
/// Defines the order in which components are processed during bumping and reset operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Precedence {
    // Field-based precedence
    Epoch,
    Major,
    Minor,
    Patch,
    PreReleaseLabel,
    PreReleaseNum,
    Post,
    Dev,

    // Schema-based precedence
    Core,
    ExtraCore,
    Build,
}

/// Precedence order management with O(1) bidirectional lookup
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrecedenceOrder {
    order: IndexMap<Precedence, ()>,
}

impl PrecedenceOrder {
    /// Create a new PrecedenceOrder from a list of precedences
    pub fn from_precedences(precedences: Vec<Precedence>) -> Self {
        let order = precedences.into_iter().map(|p| (p, ())).collect();
        Self { order }
    }

    /// Create the default PEP440-based precedence order
    pub fn pep440_based() -> Self {
        Self::from_precedences(vec![
            Precedence::Epoch,
            Precedence::Major,
            Precedence::Minor,
            Precedence::Patch,
            Precedence::Core,
            Precedence::PreReleaseLabel,
            Precedence::PreReleaseNum,
            Precedence::Post,
            Precedence::Dev,
            Precedence::ExtraCore,
            Precedence::Build,
        ])
    }

    /// Get precedence by index (O(1))
    pub fn get_precedence(&self, index: usize) -> Option<&Precedence> {
        self.order
            .get_index(index)
            .map(|(precedence, _)| precedence)
    }

    /// Get index by precedence (O(1))
    pub fn get_index(&self, precedence: &Precedence) -> Option<usize> {
        self.order.get_index_of(precedence)
    }

    /// Get the length of the precedence order
    pub fn len(&self) -> usize {
        self.order.len()
    }

    /// Check if the precedence order is empty
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    /// Iterate over all precedences in order
    pub fn iter(&self) -> impl Iterator<Item = &Precedence> {
        self.order.keys()
    }

    /// Check if a precedence exists in the order
    pub fn contains(&self, precedence: &Precedence) -> bool {
        self.order.contains_key(precedence)
    }

    /// Get all precedences as a vector
    pub fn to_vec(&self) -> Vec<Precedence> {
        self.order.keys().cloned().collect()
    }
}

impl Default for PrecedenceOrder {
    fn default() -> Self {
        Self::pep440_based()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precedence_order_creation() {
        let order = PrecedenceOrder::pep440_based();
        assert!(!order.is_empty());
        assert_eq!(order.len(), 11);
    }

    #[test]
    fn test_precedence_order_lookup() {
        let order = PrecedenceOrder::pep440_based();

        // Test get_precedence
        assert_eq!(order.get_precedence(0), Some(&Precedence::Epoch));
        assert_eq!(order.get_precedence(1), Some(&Precedence::Major));
        assert_eq!(order.get_precedence(10), Some(&Precedence::Build));
        assert_eq!(order.get_precedence(11), None);

        // Test get_index
        assert_eq!(order.get_index(&Precedence::Epoch), Some(0));
        assert_eq!(order.get_index(&Precedence::Major), Some(1));
        assert_eq!(order.get_index(&Precedence::Build), Some(10));
    }

    #[test]
    fn test_precedence_order_contains() {
        let order = PrecedenceOrder::pep440_based();

        assert!(order.contains(&Precedence::Epoch));
        assert!(order.contains(&Precedence::Major));
        assert!(order.contains(&Precedence::Build));
    }

    #[test]
    fn test_precedence_order_iter() {
        let order = PrecedenceOrder::pep440_based();
        let precedences: Vec<_> = order.iter().collect();

        assert_eq!(precedences.len(), 11);
        assert_eq!(precedences[0], &Precedence::Epoch);
        assert_eq!(precedences[1], &Precedence::Major);
        assert_eq!(precedences[10], &Precedence::Build);
    }

    #[test]
    fn test_precedence_order_to_vec() {
        let order = PrecedenceOrder::pep440_based();
        let vec = order.to_vec();

        assert_eq!(vec.len(), 11);
        assert_eq!(vec[0], Precedence::Epoch);
        assert_eq!(vec[1], Precedence::Major);
        assert_eq!(vec[10], Precedence::Build);
    }

    #[test]
    fn test_custom_precedence_order() {
        let custom_order = PrecedenceOrder::from_precedences(vec![
            Precedence::Major,
            Precedence::Minor,
            Precedence::Patch,
        ]);

        assert_eq!(custom_order.len(), 3);
        assert_eq!(custom_order.get_precedence(0), Some(&Precedence::Major));
        assert_eq!(custom_order.get_precedence(1), Some(&Precedence::Minor));
        assert_eq!(custom_order.get_precedence(2), Some(&Precedence::Patch));
        assert_eq!(custom_order.get_precedence(3), None);
    }

    #[test]
    fn test_default_precedence_order() {
        let default_order = PrecedenceOrder::default();
        let pep440_order = PrecedenceOrder::pep440_based();

        assert_eq!(default_order.len(), pep440_order.len());
        assert_eq!(default_order.to_vec(), pep440_order.to_vec());
    }
}
