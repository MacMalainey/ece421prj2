use crate::tree::*;
use crate::ops::bst_rotate;

/// Inspector for checking and manipulating a tree's state
/// 
/// Handle for manipulating and inspecting a subtree
/// Exposes methods to check the tree state, check and update balance states
/// and perform rotation operations around the top node
pub struct NodeInspector<T: Ord, U: TreeBalance>(TreeBranch<T, U>);

/// Inspector for checking a child node in a subtree that a [NodeInspector] exposes
pub struct BranchInspector<'a, T: Ord, U: TreeBalance>{
    parent_ref: std::cell::Ref<'a, TreeNode<T, U>>,
    path: TreePath
}

/// Offset descriptor for tree traversal relative to a given node
pub enum NodeOffset {
    // Traverse to root
    Root,
    // Traverse to parent
    Parent,
    // Traverse to child at path
    Child(TreePath)
}

/// Position of a node in the tree as described by a given node and a provided offset
pub struct TreePosition<T: Ord, U: TreeBalance>(TreeBranch<T, U>, NodeOffset);

/// Consumes and returns the data that the TreePosition wrapped
impl <T: Ord, U: TreeBalance> TreePosition<T, U> {
    pub fn into_inner(self) -> (TreeBranch<T, U>, NodeOffset) {
        (self.0, self.1)
    }
}

/// Trait for performing inspection operations on a given node
pub trait InspectNode<'a, T: Ord, U: TreeBalance>
    where Self: std::marker::Sized {

    /// Returns an inspector for the given child is there is one
    fn inspect_child(&'a self, path: TreePath) -> Option<BranchInspector<'a, T, U>>;

    /// Calls a function to inspect the [TreeBalance] for the node
    /// 
    /// Passes the node's [TreeBalance] into a given function and returns the result
    fn inspect_balance<F, R>(&self, apply: F) -> R where F: FnOnce(&U) -> R;

    /// Applies an operation on the [TreeBalance] for the node
    /// 
    /// Passes the node's [TreeBalance] as a mutable reference
    /// into a given function and returns the result
    fn update_balance<F, R>(&mut self, apply: F) -> R where F: FnOnce(&mut U) -> R;

    /**
     * Design note:
     * 
     * The original plan was to typedef/newtype Ref<U> and RefMut<U> types and return those
     * inspecting/updating the tree balances (if in a wrapped newtype, have it deref to Ref<U>, RefMut<U>)
     * 
     * Instead we are using functions that we pass in to apply.
     * Reason for this is it was easier to write in the long run and it was less likely
     * to cause errors.  Although the first design was attempted, it became to time consuming
     * to attempt to implement (due to lack of experience working with lifetimes)
     * 
     * If we had budgeted more time to work on it, we could have made a nicer interface to use for working with balances
     * (There are a few things that could be made more extensible or more robust concerning the InspectNode trait and it's types)
     */

    /// Returns the height of the given node
    fn inspect_height(&self) -> usize;

    /// Returns the leaf count of the given node
    fn inspect_leaves(&self) -> usize;
}

impl <'a, T: Ord, U: TreeBalance> BranchInspector<'a, T, U> {

    /// Returns a reference to the branch that his node represents
    fn get_branch(&self) -> &TreeBranch<T, U> {
        self.parent_ref.get_child(self.path).unwrap()
    }

}

impl <T: Ord, U: TreeBalance> NodeInspector<T, U> {

    /// Constructor for creating a NodeInspector
    /// for a subtree starting at the given node
    pub fn from(branch: TreeBranch<T, U>) -> NodeInspector<T, U>{
        NodeInspector (branch)
    }

    /// Performs a rotate around the root of the subtree with the given case
    /// 
    /// Note: paths are flipped.  This is to make it simpler to handle performing a rotation
    /// based off the parent and child paths of a node (i.e. to perform an outer Right rotation, )
    pub fn rotate(self, case: (TreePath, TreePath)) -> NodeInspector<T, U> {
        if case.0 != case.1 {
            let around = std::rc::Rc::clone(self.0.borrow().get_child(case.0).unwrap());
            bst_rotate(around, case.1);
        }
        NodeInspector (bst_rotate(self.0, case.0))
    }

    /// Checks if the given node is a root node
    pub fn inspect_is_root(&self) -> bool {
        self.0.borrow().get_parent().is_none()
    }

    /// Consumed the inspector returning a position along the full tree
    /// relative to the root of the subtree this inspector exposed
    pub fn into_position(self, pos: NodeOffset) -> TreePosition<T, U> {
        TreePosition (self.0, pos)
    }

}

impl <'a, T: Ord, U: TreeBalance> InspectNode<'a, T, U> for NodeInspector<T, U> {

    fn inspect_child(&'a self, path: TreePath) -> Option<BranchInspector<'a, T, U>> {
        let pref = self.0.borrow();
        if pref.get_child(path).is_some() {
            Some(BranchInspector {
                parent_ref: pref,
                path
            })
        } else {
            None
        }
    }

    fn inspect_balance<F, R>(&self, apply: F) -> R where F: FnOnce(&U) -> R {
        apply(&self.0.borrow().get_balance())
    }

    fn update_balance<F, R>(&mut self, apply: F) -> R where F: FnOnce(&mut U) -> R {
        apply(&mut self.0.borrow_mut().get_balance_mut())
    }

    fn inspect_height(&self) -> usize {
        self.0.borrow().get_height()
    }

    fn inspect_leaves(&self) -> usize {
        self.0.borrow().get_leaves()
    }

}

impl <'a, T: Ord, U: TreeBalance> InspectNode<'a, T, U> for BranchInspector<'a, T, U> {
    fn inspect_child(&'a self, path: TreePath) -> Option<BranchInspector<'a, T, U>> {
        let pref = self.get_branch().borrow();
        if pref.get_child(path).is_some() {
            Some(BranchInspector {
                parent_ref: pref,
                path
            })
        } else {
            None
        }
    }

    fn inspect_balance<F, R>(&self, apply: F) -> R where F: FnOnce(&U) -> R {
        apply(self.get_branch().borrow().get_balance())
    }

    fn update_balance<F, R>(&mut self, apply: F) -> R where F: FnOnce(&mut U) -> R {
        apply(&mut self.get_branch().borrow_mut().get_balance_mut())
    }

    fn inspect_height(&self) -> usize {
        let branch = self.get_branch();
        branch.borrow().get_height()
    }

    fn inspect_leaves(&self) -> usize {
        self.get_branch().borrow().get_leaves()
    }

}

