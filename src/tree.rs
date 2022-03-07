use std::rc::{Rc, Weak};
use std::fmt::Display;
use std::fmt::Debug;
use std::cell::RefCell;

mod ops;
pub mod inspect;
mod node;

use node::TreeNode;
use inspect::TreeBalance;

/// Enum for describing the path from one node to its child
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TreePath {
    Left,
    Right
}

use TreePath::*;
impl TreePath {

    /// Reflects a path (returns the opposite direction)
    /// 
    /// ```
    /// use project2::tree::TreePath;
    /// 
    /// // Returns the opposite path
    /// assert_eq!(TreePath::Left.reflect(), TreePath::Right);
    /// assert_eq!(TreePath::Right.reflect(), TreePath::Left);
    /// ```
    pub fn reflect(&self) -> TreePath {
        match self {
            Left => Right,
            Right => Left
        }
    }

}

/// "Balanced" binary tree implemenation
/// 
/// Uses associated [TreeBalance] to perform balancing, the tree itself does not ensure any for of balancing
/// Instead it relies on the balancing instructions provided by the [TreeBalance].
/// 
/// Using a [TreeBalance] that doesn't do any balancing will result in just an ordinary binary tree
pub struct Tree<T: Ord, U: TreeBalance>(Option<TreeBranch<T, U>>);
impl <T: Ord, U: TreeBalance> Tree<T, U> {

    /// Creates a new empty tree
    /// 
    /// ```
    /// use project2::tree::Tree;
    /// use project2::avl::AVLBalance;
    /// let tree = Tree::<usize, AVLBalance>::new();
    /// assert!(tree.is_empty());
    /// ```
    pub fn new() -> Self {
        Tree::<T, U>(None)
    }

    /// Creates a new tree, wraping the given [TreeBranch]
    fn new_with(branch: TreeBranch<T, U>) -> Self {
        Tree(Some(branch))
    }

    /// Returns true if the tree has no contents
    /// 
    /// ```
    /// use project2::tree::Tree;
    /// use project2::avl::AVLBalance;
    /// let mut tree = Tree::<usize, AVLBalance>::new();
    /// assert!(tree.is_empty());
    /// // Insert 2
    /// tree.insert(2);
    /// assert!(!tree.is_empty());
    /// // Remove 2
    /// tree.delete(&2);
    /// assert!(tree.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    /// Inserts an element into the tree
    /// 
    /// Rebalances the tree after insertion using the instructions
    /// provided by the associated [TreeBalance] type
    /// 
    /// ```
    /// use project2::tree::Tree;
    /// use project2::avl::AVLBalance;
    /// let mut tree = Tree::<usize, AVLBalance>::new();
    /// // Insert 2
    /// tree.insert(2);
    /// assert!(tree.search(&2));
    /// ```
    pub fn insert(&mut self, key: T) {
        *self = ops::bst_insert(std::mem::replace(self, Tree::new()), key)
    }

    /// Finds if an element exists in the tree
    /// 
    /// ```
    /// # use project2::tree::Tree;
    /// # use project2::avl::AVLBalance;
    /// let mut tree = Tree::<usize, AVLBalance>::new();
    /// 
    /// assert!(!tree.search(&2));
    /// 
    /// // Insert 2
    /// tree.insert(2);
    /// assert!(tree.search(&2));
    /// 
    /// // Delete 2
    /// tree.delete(&2);
    /// assert!(!tree.search(&2));
    /// ```
    pub fn search(&self, key: &T) -> bool {
        ops::bst_search(&self, key)
    }

    /// Removes an element from the tree if it exists
    /// 
    /// Removes the referenced element from the tree,
    /// returning the owned version if it exists.
    /// If delete operation is successful the tree is rebalanced
    /// using the instructions provided from the associated [TreeBalance]
    /// 
    /// ```
    /// use project2::tree::Tree;
    /// use project2::avl::AVLBalance;
    /// let mut tree = Tree::<usize, AVLBalance>::new();
    /// 
    /// assert!(tree.is_empty());
    /// 
    /// // Insert 2
    /// tree.insert(2);
    /// assert!(tree.search(&2));
    /// 
    /// // Delete 2
    /// tree.delete(&2);
    /// assert!(!tree.search(&2) && tree.is_empty());
    /// ```
    pub fn delete(&mut self, key: &T) -> Option<T> {
        let (root, key) = ops::bst_delete(std::mem::replace(self, Tree::new()), &key);
        *self = root;
        key
    }

    /// Get the height of the tree
    /// 
    /// Returns the length of the longest path from the root node to any leaf node
    /// 
    /// ```
    /// use project2::tree::Tree;
    /// use project2::avl::AVLBalance;
    /// let mut tree = Tree::<usize, AVLBalance>::new();
    /// 
    /// assert_eq!(tree.height(), 0);
    /// 
    /// // Insert 2
    /// tree.insert(2);
    /// assert_eq!(tree.height(), 1);
    /// 
    /// // Insert 3
    /// tree.insert(3);
    /// assert_eq!(tree.height(), 2);
    /// ```
    pub fn height(&self) -> usize {
        self.0.as_ref().map_or(0, |node| node.borrow().get_height())
    }

    /// Get the number of leaves of the tree
    /// 
    /// Returns the number of leaf nodes (nodes with no childen) in the tre
    /// 
    /// ```
    /// use project2::tree::Tree;
    /// use project2::avl::AVLBalance;
    /// let mut tree = Tree::<usize, AVLBalance>::new();
    /// 
    /// assert_eq!(tree.leaves(), 0);
    /// 
    /// // Insert 2
    /// tree.insert(2);
    /// assert_eq!(tree.leaves(), 1);
    /// 
    /// // Insert 3
    /// tree.insert(3);
    /// assert_eq!(tree.leaves(), 1);
    /// 
    /// // Insert 1
    /// tree.insert(1);
    /// assert_eq!(tree.leaves(), 2);
    /// ```
    pub fn leaves(&self) -> usize {
        self.0.as_ref().map_or(0, |node| node.borrow().get_leaves())
    }

    /// Clears the contents of the tree
    /// 
    /// ```
    /// use project2::tree::Tree;
    /// use project2::avl::AVLBalance;
    /// let mut tree = Tree::<usize, AVLBalance>::new();
    /// 
    /// // Insert some nodes
    /// tree.insert(2);
    /// tree.insert(3);
    /// tree.insert(1);
    /// assert_eq!(tree.height(), 2);
    /// 
    /// // Insert 1
    /// tree.clear();
    /// assert_eq!(tree.height(), 0);
    /// ```
    pub fn clear(&mut self) {
        self.0 = None;
    }

    /// Get a reference to the branch this wraps
    /// 
    /// Returns a reference to the [TreeBranch] that this tree wraps
    /// to make traversal more convient when performing operations on the tree
    fn branch(&self) -> Option<&TreeBranch<T, U>> {
        self.0.as_ref()
    }

    /// Get the branch this wraps
    /// 
    /// Returns the [TreeBranch] that this [Tree] wraps
    /// consuming the [Tree] in the process
    fn into_inner(self) -> Option<TreeBranch<T, U>> {
        self.0
    }

}


/// Shorthand type for pointer to a shared [TreeNode]
type TreeBranch<T, U> = Rc<RefCell<TreeNode<T, U>>>;
/// Shorthand type for pointer to a parent [TreeNode]
type TreeTrunk<T, U> = Weak<RefCell<TreeNode<T, U>>>;

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
            None => write!(f, "Tree: Empty")
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

// Private traits to expose needed functions from [inspect] to [ops] but not outside of the crate.
//
// Could refactor to remove the requirement for these
// but I prefere this module structure and this is a small
// amount of code bloat required to maintain it.

// Unwrapper for TreePosition
trait IntoInner {
    type Target;
    fn into_inner(self) -> Self::Target;
}

// Constructor for NodeInspector
trait Open {
    type Target;
    fn open(target: Self::Target) -> Self;
}