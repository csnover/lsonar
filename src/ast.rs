pub mod node;
pub mod quantifier;

pub use self::{node::AstNode, quantifier::Quantifier};

#[derive(Debug, Default)]
pub struct AstRoot {
    tree: Vec<AstNode>,
    capture_count: usize,
}

impl AstRoot {
    #[must_use]
    pub(crate) fn new(tree: Vec<AstNode>, capture_count: usize) -> Self {
        Self {
            tree,
            capture_count,
        }
    }

    #[must_use]
    pub fn capture_count(&self) -> usize {
        self.capture_count
    }
}

impl std::ops::Deref for AstRoot {
    type Target = [AstNode];

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}

impl PartialEq<&[AstNode]> for AstRoot {
    fn eq(&self, other: &&[AstNode]) -> bool {
        self.tree == *other
    }
}
