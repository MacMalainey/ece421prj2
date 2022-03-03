use std::fmt::Debug;
use super::tree::*;
use super::tree::TreeDir::*;

use RBColor::*;
#[derive(Clone, Copy, PartialEq, Debug)]
enum RBColor {
    Red,
    Black
}

pub struct RedBlackBalance(RBColor);

impl <T: Ord> TreeBalance<T> for RedBlackBalance {
    fn rebalance(&mut self, node: &TreeNode<T, Self>, op: TreeOp) -> Option<(TreeDir, TreeDir)> {
        match op {
            TreeOp::Insert(ppath, xpath) => {
                let pnode = node.get_child(ppath).as_ref().unwrap();
                let p = pnode.borrow();
                if p.update_balance(|b| b.0) == Red {
                    let xcolor = p.get_child(xpath).as_ref().unwrap().borrow().update_balance(|b| b.0);
                    if xcolor == Red {
                        self.0 = Red;
                        p.update_balance(|b| b.0 = Black);
                        return node.get_child(ppath.reflect()).as_ref()
                            .map_or(
                                Some((ppath, xpath)),
                                |unode| {
                                    let u = unode.borrow();
                                    match { u.update_balance(|b| b.0) } {
                                        Black => Some((ppath, xpath)),
                                        Red => {
                                            u.update_balance(|b| b.0 = Black);
                                            None
                                        }
                                    }
                                }
                            )
                    }
                }
                None
            },
            // For deletion we treat the definitions as flipped (parent path becomes longer path)
            TreeOp::Delete(upath) => {
                todo!()
            }
        }
    }

    fn new() -> Self {
        RedBlackBalance(Red)
    }

    fn mark_root(&mut self) {
        self.0 = Black
    }
}

impl Debug for RedBlackBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.0)
    }
}