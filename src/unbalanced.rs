use std::fmt::Debug;

use crate::tree::TreePath;
use crate::tree::inspect::*;

/// Implementation of an unbalanced balance
/// This effectively takes a [Tree] and makes it a regular binary search tree
pub struct UnbalancedBalance();
impl TreeBalance for UnbalancedBalance {

    fn rebalance_insert<T: Ord>(node: NodeInspector<T, Self>, _: (TreePath, TreePath)) -> TreePosition<T, Self> {
        node.into_position(NodeOffset::Root)
    }

    fn new() -> Self {
        UnbalancedBalance()
    }

    fn new_root() -> Self {
        UnbalancedBalance()
    }

    fn rebalance_delete<T: Ord>(node: NodeInspector<T, Self>, _: TreePath, _: &Self) -> TreePosition<T, Self> {
        node.into_position(NodeOffset::Root)
    }

    fn adjust_root(&mut self) {
        // Do nothing
    }
}

impl Debug for UnbalancedBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "None")
    }
}
#[cfg(test)]
mod tests {
    use super::UnbalancedBalance;
    use crate::Tree;

    #[test]
    fn insert() {
        let mut tree: Tree<u32, UnbalancedBalance> = Tree::new();

        tree.insert(40);
        tree.insert(50);
        tree.insert(60);
        assert_eq!(tree.height(), 3);
    }

    #[test]
    fn delete() {
        let mut tree: Tree<u32, UnbalancedBalance> = Tree::new();

        tree.insert(40);
        tree.insert(50);
        tree.insert(60);
        tree.insert(70);
        tree.delete(&40);
        assert_eq!(tree.height(), 3)
    }
}