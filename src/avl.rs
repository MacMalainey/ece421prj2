use std::fmt::Debug;
use super::tree::*;
use super::tree::TreeDir::*;

pub struct AVLBalance();

impl <T: Ord> TreeBalance<T> for AVLBalance {
    fn rebalance(&mut self, node: &TreeNode<T, Self>, op: TreeOp) -> Option<(TreeDir, TreeDir)> {
        match op {
            TreeOp::Insert(ppath, xpath) => {
                let rebalance = {
                    avl_check(node.get_child(ppath), node.get_child(ppath.reflect()))
                };
    
                if rebalance {
                    Some((ppath, xpath))
                } else {
                    None
                }
            },
            // For deletion we treat the definitions as flipped (parent path becomes longer path)
            TreeOp::Delete(upath) => {
                let ppath = upath.reflect();
                let rebalance = {
                    avl_check(node.get_child(ppath), node.get_child(upath))
                };
                if rebalance {
                    let r = node;
                    let pnode = r.get_child(ppath).as_ref().unwrap();
                    let xpath = {
                        let p = pnode.borrow();
                        p.get_child(Left)
                            .as_ref()
                            .map_or(
                                Right,
                                |xlnode| {
                                    let xr_height = p.get_child(Right).as_ref().map_or(0, |xrnode| xrnode.borrow().get_height());
                                    if xr_height > xlnode.borrow().get_height() {
                                        Right
                                    } else {
                                        Left
                                    }
                                }
                            )
                    };

                    Some((ppath, xpath))
                } else {
                    None
                }
            }
        }
    }

    fn new() -> Self {
        AVLBalance()
    }

    fn mark_root(&mut self) {}
}   

pub fn avl_check<T: Ord>(parent: &Option<TreeBranch<T, AVLBalance>>, uncle: &Option<TreeBranch<T, AVLBalance>>) -> bool {
    let uheight = uncle.as_ref().map_or(0, |node| node.borrow().get_height());
    let pheight = parent.as_ref().map_or(0, |node| node.borrow().get_height());

    pheight - uheight > 1
}

impl Debug for AVLBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "AVL")
    }
}