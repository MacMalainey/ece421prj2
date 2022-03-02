use std::fmt::Display;
use std::fmt::Debug;
use std::cell::RefCell;
use std::rc::Rc;

use super::tree::*;
use super::tree::TreeDir::*;

pub type AVLBranch<T> = Rc<RefCell<TreeNode<T>>>;
pub type AVLTree<T> = Option<AVLBranch<T>>;

pub fn avl_check<T: Ord>(parent: &AVLTree<T>, uncle: &AVLTree<T>) -> bool {
    let uheight = uncle.as_ref().map_or(0, |node| node.borrow().get_height());
    let pheight = parent.as_ref().map_or(0, |node| node.borrow().get_height());

    pheight - uheight > 1
}

pub fn avl_balance<T: Ord>(rnode: &AVLBranch<T>, op: TreeOp) {
    match op {
        TreeOp::Insert(ppath, xpath) => {
            let rebalance = {
                avl_check(rnode.borrow().get_child(ppath), rnode.borrow().get_child(ppath.reflect()))
            };

            if rebalance {
                if xpath != ppath {
                    let r = rnode.borrow();
                    let pnode = r.get_child(ppath).as_ref().unwrap();
                    helper::bst_rotate(pnode, xpath);
                }
    
                helper::bst_rotate(rnode, ppath);
            }
        },
        // For deletion we treat the definitions as flipped (parent path becomes longer path)
        TreeOp::Delete(upath) => {
            let ppath = upath.reflect();
            let rebalance = {
                avl_check(rnode.borrow().get_child(ppath), rnode.borrow().get_child(upath))
            };
            if rebalance {
                {
                    let r = rnode.borrow();
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

                    if xpath != ppath {
                        helper::bst_rotate(pnode, xpath);
                    }
                }
                helper::bst_rotate(rnode, ppath);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Tree;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn manual_check_order() {
        let nums = [
            40, 65, 55,
            57, 58, 75, 60, 59
        ];
        let mut tree: Tree<u32> = Tree::new();

        for num in &nums {
            println!("----- Insert: {} -------", num);
            tree.insert(*num);
            println!("{:#?}", tree);
        }
    }

    #[test]
    fn manual_check_delete() {
        let nums = [
            40, 65, 55,
            57, 58, 75, 60, 59
        ];
        let mut tree: Tree<u32> = Tree::new();

        for num in &nums {
            tree.insert(*num);
        }
        println!("----- Insert: {} -------", 40);
        assert_eq!(tree.delete(&58), Some(58));
        println!("{:#?}", tree);
    }
}