use std::rc::Rc;
use std::fmt::Display;
use std::fmt::Debug;
use std::cell::RefCell;
use std::cmp::{Ordering, max};

use crate::ops::*;
use crate::inspect::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TreePath {
    Left,
    Right
}

use TreePath::*;
impl TreePath {

    pub fn reflect(&self) -> TreePath {
        match self {
            Left => Right,
            Right => Left
        }
    }

}

pub struct Tree<T: Ord, U: TreeBalance>(Option<TreeBranch<T, U>>);

impl <T: Ord, U: TreeBalance> Tree<T, U> {
    pub fn new() -> Self {
        Tree::<T, U>(None)
    }

    pub fn new_with(branch: TreeBranch<T, U>) -> Self {
        Tree(Some(branch))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    pub fn insert(&mut self, key: T) {
        *self = bst_insert(self.clone_branch(), key)
    }

    pub fn delete(&mut self, key: &T) -> Option<T> {
        let (root, key) = bst_delete(self.clone_branch(), &key);
        *self = root;
        key
    }

    pub fn height(&self) -> usize {
        self.0.as_ref().map_or(0, |node| node.borrow().get_height())
    }

    pub fn leaves(&self) -> usize {
        self.0.as_ref().map_or(0, |node| node.borrow().get_leaves())
    }

    pub fn branch(&self) -> Option<&TreeBranch<T, U>> {
        self.0.as_ref()
    }

    pub fn into_inner(self) -> Option<TreeBranch<T, U>> {
        self.0
    }

    fn clone_branch(&self) -> Self {
        self.branch().map_or(Tree::new(), |b| Tree::new_with(Rc::clone(&b)))
    }

}

pub type TreeBranch<T, U> = Rc<RefCell<TreeNode<T, U>>>;

pub trait TreeBalance
    where Self: std::marker::Sized
{
    fn new() -> Self;
    fn new_root() -> Self;
    fn rebalance_insert<T: Ord>(inspector: NodeInspector<T, Self>, path: (TreePath, TreePath)) -> NextTreePosition<T, Self>;
    fn rebalance_delete<T: Ord>(inspector: NodeInspector<T, Self>, path: TreePath, balance: &Self) -> NextTreePosition<T, Self>;
}

pub struct TreeNode<T: Ord, U>
    where U: TreeBalance
{
    key: T,
    height: usize,
    leaves: usize,
    parent: Tree<T, U>,
    left: Tree<T, U>,
    right: Tree<T, U>,
    balance: U
}

impl <T: Ord, U: TreeBalance> TreeNode<T, U> {
    pub fn new_with(key: T) -> Self {
        TreeNode {
            key,
            height: 1,
            leaves: 1,
            parent: Tree::new(),
            left: Tree::new(),
            right: Tree::new(),
            balance: U::new_root()
        }
    }

    pub fn new_with_parent(key: T, parent: Tree<T, U>) -> Self {
        TreeNode {
            key,
            height: 1,
            leaves: 1,
            parent,
            left: Tree::new(),
            right: Tree::new(),
            balance: U::new()
        }
    }

    pub fn update(&mut self) {
        self.update_leaves();
        self.update_height();
    }

    fn update_leaves(&mut self) {
        self.leaves = max(
            1,
            self.left.0.as_ref().map_or(0, |node| node.borrow().get_leaves())
                + self.right.0.as_ref().map_or(0, |node| node.borrow().get_leaves())
        )
    }

    fn update_height(&mut self) {
        self.height = max(
            self.left.0.as_ref().map_or(0, |node| node.borrow().height),
            self.right.0.as_ref().map_or(0, |node| node.borrow().height)
        ) + 1;
    }

    pub fn get_child(&self, pos: TreePath) -> Option<&TreeBranch<T, U>> {
        match pos {
            Left => self.left.0.as_ref(),
            Right => self.right.0.as_ref()
        }
    }

    pub fn get_balance(&self) -> &U {
        &self.balance
    }

    pub fn get_balance_mut(&mut self) -> &mut U {
        &mut self.balance
    }

    pub fn get_parent(&self) -> Option<&TreeBranch<T, U>> {
        self.parent.0.as_ref()
    }

    pub fn get_joint(&mut self, pos: TreePath) -> &mut Tree<T, U> {
        match pos {
            Left => &mut self.left,
            Right => &mut self.right
        }
    }

    pub fn get_parent_joint(&mut self) -> &mut Tree<T, U> {
        &mut self.parent
    }

    pub fn prune(&mut self, pos: TreePath) -> Tree<T, U> {
        // Perform replacement
        let pruned = match pos {
            Left => std::mem::replace(&mut self.left, Tree::new()),
            Right => std::mem::replace(&mut self.right, Tree::new())
        };

        // Remove parent reference from the path we just detached
        if let Some(ref p) = pruned.0 {
            *p.borrow_mut().get_parent_joint() = Tree::new()
        }

        pruned
    }

    pub fn search(&self, key: &T) -> Option<TreePath> {
        match key.cmp(&self.key) {
            Ordering::Equal => None,
            Ordering::Greater => Some(Right),
            Ordering::Less => Some(Left)
        }
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn get_leaves(&self) -> usize {
        self.leaves
    }

    pub fn find_placement(&self, child: &TreeNode<T, U>) -> TreePath {
        self.search(&child.key).unwrap()
    }

    pub fn pop(self) -> (T, U) {
        (self.key, self.balance)
    }

    pub fn replace_key(&mut self, mut key: T) -> T {
        if let Some(ref node) = self.left.0 {
            let n = node.borrow();
            assert_ne!(key.cmp(&n.key), Ordering::Less);
        }

        if let Some(ref node) = self.right.0 {
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

impl <T, U> Display for Tree<T, U>
where
    T: Ord + Display,
    U: TreeBalance
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match &self.0 {
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
    U: TreeBalance + Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match &self.0 {
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
    U: TreeBalance
{

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self.left.0 {
            Some(ref node) => std::fmt::Display::fmt(&node.borrow(), f),
            None => Ok(())
        }.and_then(|_| write!(f, "{}, ", self.key)).and_then(|_|
            match self.right.0 {
                Some(ref node) => std::fmt::Display::fmt(&node.borrow(), f),
                None => Ok(())
            }
        )
    }
}

impl <T, U> Debug for TreeNode<T, U>
where
    T: Ord + Debug,
    U: TreeBalance + Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let mut builder = f.debug_struct(&format!("{:?}", &self.key));
        builder.field("balance", &self.balance);
        builder.field("height", &self.height);
        builder.field("leaves", &self.leaves);
        match self.right.0 {
            Some(ref node) => builder.field("right", &node.borrow()),
            None => builder.field("right", &"None")
        };
        match self.left.0 {
            Some(ref node) => builder.field("left", &node.borrow()),
            None => builder.field("left", &"None")
        };
        builder.finish()
    }
}