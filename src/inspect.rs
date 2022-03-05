use crate::tree::*;
use crate::ops::bst_rotate;

pub struct NodeInspector<T: Ord, U: TreeBalance>(TreeBranch<T, U>);

pub struct BranchInspector<'a, T: Ord, U: TreeBalance>{
    parent_ref: std::cell::Ref<'a, TreeNode<T, U>>,
    path: TreePath
}

pub enum TreePosition {
    Root,
    Parent,
    Child(TreePath)
}

pub struct NextTreePosition<T: Ord, U: TreeBalance>(TreeBranch<T, U>, TreePosition);

impl <T: Ord, U: TreeBalance> NextTreePosition<T, U> {
    pub fn into_inner(self) -> (TreeBranch<T, U>, TreePosition) {
        (self.0, self.1)
    }
}

pub trait InspectNode<'a, T: Ord, U: TreeBalance>
    where Self: std::marker::Sized {
    fn inspect_child(&'a self, path: TreePath) -> Option<BranchInspector<'a, T, U>>;
    fn inspect_balance<F, R>(&self, apply: F) -> R where F: FnOnce(&U) -> R;
    fn update_balance<F, R>(&mut self, apply: F) -> R where F: FnOnce(&mut U) -> R;
    fn inspect_height(&self) -> usize;
    fn inspect_leaves(&self) -> usize;
}

impl <'a, T: Ord, U: TreeBalance> BranchInspector<'a, T, U> {

    fn get_branch(&self) -> &TreeBranch<T, U> {
        self.parent_ref.get_child(self.path).unwrap()
    }

}

impl <T: Ord, U: TreeBalance> NodeInspector<T, U> {

    pub fn from(branch: TreeBranch<T, U>) -> NodeInspector<T, U>{
        NodeInspector (branch)
    }

    pub fn rotate(self, case: (TreePath, TreePath)) -> NodeInspector<T, U> {
        if case.0 != case.1 {
            let around = std::rc::Rc::clone(self.0.borrow().get_child(case.0).unwrap());
            bst_rotate(around, case.1);
        }
        NodeInspector (bst_rotate(self.0, case.0))
    }

    pub fn inspect_is_root(&self) -> bool {
        self.0.borrow().get_parent().is_none()
    }

    pub fn into_next_position(self, pos: TreePosition) -> NextTreePosition<T, U> {
        NextTreePosition (self.0, pos)
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

