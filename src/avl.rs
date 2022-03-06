use std::fmt::Debug;

use crate::tree::TreePath;
use crate::tree::inspect::*;

/// Implementation of a balance for a binary tree
/// that effectively converts a [Tree] into into an AVLTree
pub struct AVLBalance();
impl TreeBalance for AVLBalance {

    fn rebalance_insert<T: Ord>(node: NodeInspector<T, Self>, path: (TreePath, TreePath)) -> TreePosition<T, Self> {
        // Check if we need to rebalance
        let rebalance = {
            let pheight = node.inspect_child(path.0).unwrap().inspect_height();
            let uheight = node.inspect_child(path.0.reflect()).map_or(0, |b| b.inspect_height());

            pheight - uheight > 1
        };

        if rebalance {
            node.rotate(path).into_position(NodeOffset::Root)
        } else {
            node.into_position(NodeOffset::Parent)
        }

    }

    fn new() -> Self {
        AVLBalance()
    }

    fn new_root() -> Self {
        AVLBalance()
    }

    fn rebalance_delete<T: Ord>(node: NodeInspector<T, Self>, upath: TreePath, _: &Self) -> TreePosition<T, Self> {
        // Check if we need to rebalance
        let ppath = upath.reflect();
        let rebalance = {
            let pheight = node.inspect_child(ppath).map_or(0, |b| b.inspect_height());
            let uheight = node.inspect_child(upath).map_or(0, |b| b.inspect_height());

            pheight - uheight > 1
        };

        if rebalance {
            // Get child path that we need for the rebalance
            let xpath = {
                let pnode = node.inspect_child(ppath).unwrap();
                let inline_height = pnode.inspect_child(ppath).map_or(0, |x| x.inspect_height());
                let elbow_height = pnode.inspect_child(ppath.reflect()).map_or(0, |x| x.inspect_height());

                // Optimize selection to prevent unneccessary rotation
                if inline_height >= elbow_height {
                    ppath
                } else {
                    ppath.reflect()
                }
            };
            node.rotate((ppath, xpath)).into_position(NodeOffset::Parent)
        } else {
            node.into_position(NodeOffset::Parent)
        }
    }

    fn adjust_root(&mut self) {
        // Do nothing
    }
}

impl Debug for AVLBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "AVL")
    }
}

#[cfg(test)]
mod tests {
    use super::AVLBalance;
    use crate::Tree;

    #[test]
    fn insert() {
        let mut tree: Tree<u32, AVLBalance> = Tree::new();

        tree.insert(40);
        tree.insert(50);
        tree.insert(60);
        assert_eq!(tree.height(), 2)
    }

    #[test]
    fn delete() {
        let mut tree: Tree<u32, AVLBalance> = Tree::new();

        tree.insert(40);
        tree.insert(50);
        tree.insert(60);
        tree.insert(70);
        tree.delete(&40);
        assert_eq!(tree.height(), 2)
    }
}