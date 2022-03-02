use std::cell::RefCell;
use std::rc::Rc;

use super::treenew::*;
use super::treenew::TreeDir::*;

pub struct AVLBranch<T: Ord>(Rc<TreeNodeRef<T, Self>>);

impl <T: Ord> TreeBranch<T> for AVLBranch<T> {
    fn rebalance(&self, op: TreeOp) -> Option<(TreeDir, TreeDir)> {
        match op {
            TreeOp::Insert(ppath, xpath) => {
                let rebalance = {
                    avl_check(self.borrow().get_child(ppath), self.borrow().get_child(ppath.reflect()))
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
                    avl_check(self.borrow().get_child(ppath), self.borrow().get_child(upath))
                };
                if rebalance {
                    let r = self.borrow();
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
    fn new(node: TreeNode<T, Self>) -> Self {
        AVLBranch(Rc::new(RefCell::new(node)))
    }
    fn into_inner(self) -> Result<TreeNodeRef<T, Self>, Self> {
        Rc::try_unwrap(
            self.0
        ).map_err(|orig| AVLBranch(orig))
    }
}

impl <T: Ord> Clone for AVLBranch<T> {
    fn clone(&self) -> Self {
        AVLBranch(Rc::clone(&self.0))
    }
}

impl <T: Ord> std::ops::Deref for AVLBranch<T> {
    type Target=TreeNodeRef<T, AVLBranch<T>>;

    fn deref(&self) -> &<Self as std::ops::Deref>::Target {
        &self.0
    }
}

pub fn avl_check<T: Ord>(parent: &Option<AVLBranch<T>>, uncle: &Option<AVLBranch<T>>) -> bool {
    let uheight = uncle.as_ref().map_or(0, |node| node.borrow().get_height());
    let pheight = parent.as_ref().map_or(0, |node| node.borrow().get_height());

    pheight - uheight > 1
}