use std::rc::Rc;
use std::fmt::Display;
use std::fmt::Debug;
use std::cell::RefCell;
use std::cmp::{Ordering, max};

mod ops;
use ops::{bst_insert, bst_delete};

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

pub struct Tree<T: Ord, U>
where
    U: TreeBalance<T>,
{
    root: Option<TreeBranch<T, U>>
}

impl <T: Ord, U> Tree<T, U>
    where U: TreeBalance<T>
{
    pub fn new() -> Self {
        Tree::<T, U> {
            root: None
        }
    }

    pub fn insert(&mut self, key: T) {
        bst_insert(&mut self.root, key)
    }

    pub fn delete(&mut self, key: &T) -> Option<T> {
        bst_delete(&mut self.root, key)
    }

    pub fn height(&self) -> usize {
        self.root.as_ref().map_or(0, |node| node.borrow().get_height())
    }
}

pub type TreeBranch<T, U> = Rc<RefCell<TreeNode<T, U>>>;

pub trait TreeBalance<T: Ord>
    where Self: std::marker::Sized
{
    fn new() -> Self;
    fn rebalance(&mut self,
        node: &TreeNode<T, Self>,
        op: TreeOp
    ) -> Option<(TreeDir, TreeDir)>;
    fn mark_root(&mut self);
}

pub struct TreeNode<T: Ord, U>
    where U: TreeBalance<T>
{
    key: T,
    height: usize,
    left: Option<TreeBranch<T, U>>,
    right: Option<TreeBranch<T, U>>,
    balance: RefCell<U>
}

impl <T: Ord, U> TreeNode<T, U>
    where U: TreeBalance<T>
{

    fn new_with(key: T) -> Self {
        TreeNode {
            key,
            height: 1,
            left: None,
            right: None,
            balance: RefCell::new(U::new())
        }
    }

    pub fn update_height(&mut self) {
        self.height = max(
            self.left.as_ref().map_or(0, |node| node.borrow().height),
            self.right.as_ref().map_or(0, |node| node.borrow().height)
        ) + 1;
    }

    pub fn get_child(&self, pos: TreeDir) -> &Option<TreeBranch<T, U>> {
        match pos {
            Left => &self.left,
            Right => &self.right
        }
    }

    pub fn get_child_mut(&mut self, pos: TreeDir) -> &mut Option<TreeBranch<T, U>> {
        match pos {
            Left => &mut self.left,
            Right => &mut self.right
        }
    }

    pub fn prune(&mut self, pos: TreeDir) -> Option<TreeBranch<T, U>> {
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

    pub fn rebalance(&self, op: TreeOp) -> Option<(TreeDir, TreeDir)> {
        self.balance.borrow_mut().rebalance(self, op)
    }

    pub fn mark_root(&self) {
        self.balance.borrow_mut().mark_root()
    }

    pub fn update_balance<F, R>(&self, op: F) -> R
    where F: FnOnce(&mut U) -> R
    {
        op(&mut self.balance.borrow_mut())
    }

}

  /*********************/
 // FORMATTING TRAITS //
/*********************/

impl <T, U> Display for Tree<T, U>
where
    T: Ord + Display,
    U: TreeBalance<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match &self.root {
            Some(node) => {
                write!(f, "Tree: {{ {}}}", node.borrow())
            }
            None => Ok(())
        }
    }
}

impl <T, U> Debug for Tree<T, U>
where
    T: Ord + Debug,
    U: TreeBalance<T> + Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match &self.root {
            Some(node) => {
                node.borrow().fmt(f)
            }
            None => Ok(())
        }
    }
}

impl <T, U> Display for TreeNode<T, U>
where
    T: Ord + Display,
    U: TreeBalance<T>
{

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

impl <T, U> Debug for TreeNode<T, U>
where
    T: Ord + Debug,
    U: TreeBalance<T> + Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let mut builder = f.debug_struct(&format!("{:?}", &self.key));
        builder.field("balance", &self.balance.borrow());
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