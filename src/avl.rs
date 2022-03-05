use std::fmt::Debug;
use crate::tree::*;
use crate::inspect::*;

pub struct AVLBalance();

impl TreeBalance for AVLBalance {

    fn rebalance_insert<T: Ord>(node: NodeInspector<T, Self>, path: (TreePath, TreePath)) -> NextTreePosition<T, Self> {
        let rebalance = {
            let pheight = node.inspect_child(path.0).unwrap().inspect_height();
            let uheight = node.inspect_child(path.0.reflect()).map_or(0, |b| b.inspect_height());

            pheight - uheight > 1
        };

        if rebalance {
            node.rotate(path).into_next_position(TreePosition::Root)
        } else {
            node.into_next_position(TreePosition::Parent)
        }

    }

    fn new() -> Self {
        AVLBalance()
    }

    fn new_root() -> Self {
        AVLBalance()
    }

    fn rebalance_delete<T: Ord>(node: NodeInspector<T, Self>, upath: TreePath, _: &Self) -> NextTreePosition<T, Self> {
        let ppath = upath.reflect();
        let rebalance = {
            let pheight = node.inspect_child(ppath).map_or(0, |b| b.inspect_height());
            let uheight = node.inspect_child(upath).map_or(0, |b| b.inspect_height());

            pheight - uheight > 1
        };

        if rebalance {
            // Get child path
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
            node.rotate((ppath, xpath)).into_next_position(TreePosition::Parent)
        } else {
            node.into_next_position(TreePosition::Parent)
        }
    }
}

impl Debug for AVLBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "AVL")
    }
}