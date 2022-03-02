use std::fmt::Display;
use std::fmt::Debug;
use std::cell::RefCell;
use std::rc::Rc;
use std::cmp::{Ordering, max};

use super::avl::*;

pub mod helper;
use helper::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TreeDir {
    Left,
    Right
}

use TreeDir::*;
impl TreeDir {

    pub fn reflect(&self) -> TreeDir {
        match self {
            Left => Right,
            Right => Left
        }
    }

}

pub enum TreeOp {
    Insert(TreeDir, TreeDir),
    Delete(TreeDir)
}

impl <T: Ord> Tree<T> {
    pub fn new() -> Self {
        Tree(None)
    }

    pub fn insert(&mut self, key: T) {
        bst_insert(&mut self.0, key)
    }

    pub fn delete(&mut self, key: &T) -> Option<T> {
        bst_delete(&mut self.0, key)
    }

    pub fn height(&self) -> usize {
        self.0.as_ref().map_or(0, |node| node.borrow().get_height())
    }
}

pub type TreeNodeRef<T> = Rc<RefCell<TreeNode<T>>>;

pub trait TreeBranch<T: Ord>: std::ops::Deref<Target=TreeNodeRef<T>> {
    fn rebalance(&self, op: TreeOp);
    fn rotate(&self, ppath: TreeDir, xpath: TreeDir);
}

pub struct Tree<T: Ord>(AVLTree<T>);

pub struct TreeNode<T: Ord> {
    key: T,
    height: usize,
    left: AVLTree<T>,
    right: AVLTree<T>,
}

impl <T: Ord> TreeNode<T> {

    pub fn new_with(key: T) -> Self {
        TreeNode {
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

  /*********************/
 // FORMATTING TRAITS //
/*********************/

impl <T: Ord + Display> Display for Tree<T> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match &self.0 {
            Some(node) => {
                write!(f, "Tree: {{ {}}}", node.borrow())
            }
            None => Ok(())
        }
    }
}

impl <T: Ord + Debug> Debug for Tree<T> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match &self.0 {
            Some(node) => {
                node.borrow().fmt(f)
            }
            None => Ok(())
        }
    }
}

impl <T: Ord + Display> Display for TreeNode<T> {

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

impl <T: Ord + Debug> Debug for TreeNode<T> {

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