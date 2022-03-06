use std::fmt::Debug;

use crate::tree::{TreeBalance, TreePath};
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
}

impl Debug for UnbalancedBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "None")
    }
}