use std::cmp::Ordering;

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

pub trait TreeNode<T: Ord> 
    where Self: std::marker::Sized {

    fn new_from_key(key: T) -> Self;
    fn is_empty(key: T) -> bool;
    fn pop(&mut self) -> T;
    fn prune(&mut self, pos: TreeDir) -> Option<Self>;
    fn get_child(&self, pos: TreeDir) -> Option<&Self>;
    fn get_child_mut(&mut self, pos: TreeDir) -> Option<&mut Self>;
    fn insert(&mut self, pos: TreeDir);
    fn cmp(&self, key: &T) -> Ordering;
}

pub trait BalancedTreeNode<T: Ord>: TreeNode<T> {
    
}
