use std::fmt::Display;
use std::fmt::Debug;
use std::cell::RefCell;
use std::rc::Rc;
use std::cmp::{Ordering, max};

use super::tree::*;
use super::tree::TreeDir::*;

pub type AVLBranch<T> = Rc<RefCell<AVLNode<T>>>;
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
                    bst_rotate(&pnode, xpath);
                }
    
                bst_rotate(rnode, ppath);
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
                        bst_rotate(&pnode, xpath);
                    }
                }
                bst_rotate(rnode, ppath);
            }
        }
    }
}

pub struct AVLNode<T: Ord> {
    key: T,
    height: usize,
    left: AVLTree<T>,
    right: AVLTree<T>,
}

impl <T: Ord> AVLNode<T> {

    pub fn new_with(key: T) -> Self {
        AVLNode {
            key,
            height: 1,
            left: None,
            right: None
        }
    }

    pub fn update_height(&mut self) {
        self.height = max(
            self.left.as_ref().map_or(0, |node| node.borrow().height),
            self.right.as_ref().map_or(0, |node| node.borrow().height)
        ) + 1;
    }

    pub fn get_child(&self, pos: TreeDir) -> &AVLTree<T> {
        match pos {
            Left => &self.left,
            Right => &self.right
        }
    }

    pub fn get_child_mut(&mut self, pos: TreeDir) -> &mut AVLTree<T> {
        match pos {
            Left => &mut self.left,
            Right => &mut self.right
        }
    }

    pub fn prune(&mut self, pos: TreeDir) -> AVLTree<T> {
        match pos {
            Left => self.left.take(),
            Right => self.right.take()
        }
    }

    pub fn search(&self, key: &T) -> Option<TreeDir> {
        match key.cmp(&self.key) {
            Ordering::Equal => None,
            Ordering::Greater => Some(Right),
            Ordering::Less => Some(Left)
        }
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn pop(self) -> T {
        self.key
    }

    pub fn replace_key(&mut self, mut key: T) -> T {
        if let Some(ref node) = self.left {
            let n = node.borrow();
            assert_ne!(key.cmp(&n.key), Ordering::Less);
        }

        if let Some(ref node) = self.right {
            let n = node.borrow();
            assert_ne!(key.cmp(&n.key), Ordering::Greater);
        }

        std::mem::swap(&mut self.key, &mut key);

        key
    }

}

impl <T: Ord + Display> Display for AVLNode<T> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self.left {
            Some(ref node) => std::fmt::Display::fmt(&node.borrow(), f),
            None => Ok(())
        }.and_then(|_| write!(f, "{}, ", self.key)).and_then(|_|
            match self.right {
                Some(ref node) => std::fmt::Display::fmt(&node.borrow(), f),
                None => Ok(())
            }
        )
    }
}

impl <T: Ord + Debug> Debug for AVLNode<T> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let mut builder = f.debug_struct(&format!("{:?}", &self.key));
        builder.field("height", &self.height);
        match self.right {
            Some(ref node) => builder.field("right", &node.borrow()),
            None => builder.field("right", &"None")
        };
        match self.left {
            Some(ref node) => builder.field("left", &node.borrow()),
            None => builder.field("left", &"None")
        };
        builder.finish()
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