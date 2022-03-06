use std::rc::{Rc, Weak};
use std::fmt::Display;
use std::fmt::Debug;
use std::cell::RefCell;
use std::cmp::{Ordering, max};

use crate::ops::*;
use crate::inspect::*;

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
    /// # use project2::tree::Tree;
    /// # use project2::avl::AVLBalance;
    /// let tree = Tree::<usize, AVLBalance>::new();
    /// assert!(tree.is_empty());
    /// ```
    pub fn new() -> Self {
        Tree::<T, U>(None)
    }

    /// Creates a new tree, wraping the given [TreeBranch]
    pub fn new_with(branch: TreeBranch<T, U>) -> Self {
        Tree(Some(branch))
    }

    /// Returns true if the tree has no contents
    /// 
    /// ```
    /// # use project2::tree::Tree;
    /// # use project2::avl::AVLBalance;
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
    /// # use project2::tree::Tree;
    /// # use project2::avl::AVLBalance;
    /// let mut tree = Tree::<usize, AVLBalance>::new();
    /// // Insert 2
    /// tree.insert(2);
    /// assert!(tree.search(&2));
    /// ```
    pub fn insert(&mut self, key: T) {
        *self = bst_insert(std::mem::replace(self, Tree::new()), key)
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
        bst_search(&self, key)
    }

    /// Removes an element from the tree if it exists
    /// 
    /// Removes the referenced element from the tree,
    /// returning the owned version if it exists.
    /// If delete operation is successful the tree is rebalanced
    /// using the instructions provided from the associated [TreeBalance]
    /// 
    /// ```
    /// # use project2::tree::Tree;
    /// # use project2::avl::AVLBalance;
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
        let (root, key) = bst_delete(std::mem::replace(self, Tree::new()), &key);
        *self = root;
        key
    }

    /// Get the height of the tree
    /// 
    /// Returns the length of the longest path from the root node to any leaf node
    /// 
    /// ```
    /// # use project2::tree::Tree;
    /// # use project2::avl::AVLBalance;
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
    /// # use project2::tree::Tree;
    /// # use project2::avl::AVLBalance;
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

    /// Get a reference to the branch this wraps
    /// 
    /// Returns a reference to the [TreeBranch] that this tree wraps
    /// to make traversal more convient when performing operations on the tree
    /// 
    /// ```
    /// # use project2::tree::{Tree, TreeNode};
    /// # use project2::avl::AVLBalance;
    /// let tree = Tree::<usize, AVLBalance>::new();
    /// 
    /// assert!(tree.branch().is_none());
    /// 
    /// let tree = Tree::<usize, AVLBalance>::new_with(
    ///     std::rc::Rc::new(
    ///         std::cell::RefCell::new(
    ///             TreeNode::new_with(2)
    ///         )
    ///     )
    /// );
    /// assert!(tree.branch().is_some())
    /// ```
    pub fn branch(&self) -> Option<&TreeBranch<T, U>> {
        self.0.as_ref()
    }

    /// Get the branch this wraps
    /// 
    /// Returns the [TreeBranch] that this [Tree] wraps
    /// consuming the [Tree] in the process
    /// 
    /// ```
    /// # use project2::tree::{Tree, TreeNode};
    /// # use project2::avl::AVLBalance;
    /// let tree = Tree::<usize, AVLBalance>::new();
    /// 
    /// assert!(tree.into_inner().is_none());
    /// 
    /// let tree = Tree::<usize, AVLBalance>::new_with(
    ///     std::rc::Rc::new(
    ///         std::cell::RefCell::new(
    ///             TreeNode::new_with(2)
    ///         )
    ///     )
    /// );
    /// assert!(tree.into_inner().is_some())
    /// ```
    pub fn into_inner(self) -> Option<TreeBranch<T, U>> {
        self.0
    }

}


/// Shorthand type for pointer to a shared [TreeNode]
pub type TreeBranch<T, U> = Rc<RefCell<TreeNode<T, U>>>;
/// Shorthand type for pointer to a parent [TreeNode]
pub type TreeTrunk<T, U> = Weak<RefCell<TreeNode<T, U>>>;

/// Balance trait for a [Tree]
/// 
/// The [TreeBalance] associated with a [Tree] will be used for 
/// rebalancing the tree after insert and delete operations
pub trait TreeBalance
    where Self: std::marker::Sized
{
    /// Returns a new balance for a generic node
    fn new() -> Self;

    /// Returns a new balance for a root node
    fn new_root() -> Self;

    /// Perform a rebalance after an insertion operation
    /// 
    /// Rebalance the tree starting at a given node and return the next
    /// position that should be inspected for rebalancing.
    /// The tree is inspectable through the passed [NodeInspector] object and
    /// rotations can be performed on the tree used it.
    /// The path of the operation is provided in format (Parent Insertion Path, Child Insertion Path)
    /// 
    /// Returns the next position to rebalance in relation to the node currently being balanced
    fn rebalance_insert<T: Ord>(inspector: NodeInspector<T, Self>, path: (TreePath, TreePath)) -> NextTreePosition<T, Self>;

    /// Perform a rebalance after a delete operation
    /// 
    /// Rebalance the tree starting at a given node and return the next
    /// position that should be inspected for rebalancing.
    /// The tree is inspectable through the passed [NodeInspector] object and
    /// rotations can be performed on the tree used it.
    /// The path provided is the path that the child was deleted from in relation to the current node.
    /// The balance from the deleted node is also provided.
    /// 
    /// Returns the next position to rebalance in relation to the node currently being balanced
    fn rebalance_delete<T: Ord>(inspector: NodeInspector<T, Self>, path: TreePath, balance: &Self) -> NextTreePosition<T, Self>;
}

/// A node in a binary tree structure
/// 
/// Has a key of type [T] and is balanced
/// using the associated [TreeBalance] type
pub struct TreeNode<T: Ord, U>
    where U: TreeBalance
{
    /// Key for the node
    key: T,
    /// Height of the tree that is rooted by this node
    height: usize,
    /// Number of leaves of the tree that is rooted by this node
    leaves: usize,
    /// Reference to parent node
    parent: TreeTrunk<T, U>,
    /// Reference to left child node
    left: Tree<T, U>,
    /// Reference to right child node
    right: Tree<T, U>,
    /// [TreeBalance] type to use for balancing
    balance: U
}

impl <T: Ord, U: TreeBalance> TreeNode<T, U> {
    /// Constructs a new tree node with the given key
    /// 
    /// Creates a parentless [TreeNode] that owns the given key
    /// and initializes the associated [TreeBalance] as a root node
    pub fn new_with(key: T) -> Self {
        TreeNode {
            key,
            height: 1,
            leaves: 1,
            parent: Weak::new(),
            left: Tree::new(),
            right: Tree::new(),
            balance: U::new_root()
        }
    }

    /// Constructs a new tree node with the given key and parent
    /// 
    /// Creates [TreeNode] that owns the given key and references the given parent
    /// and initializes the associated [TreeBalance] as a non-root node (even if parent reference is None)
    pub fn new_with_parent(key: T, parent: TreeTrunk<T, U>) -> Self {
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

    /// Update the node's knowledge of the tree
    /// 
    /// Updates the node's cached information regarding its' tree
    /// structure (height and number of leaves)
    pub fn update(&mut self) {
        self.update_leaves();
        self.update_height();
    }

    /// Updates the node's counter the tree's leaves
    fn update_leaves(&mut self) {
        self.leaves = max(
            1,
            self.left.0.as_ref().map_or(0, |node| node.borrow().get_leaves())
                + self.right.0.as_ref().map_or(0, |node| node.borrow().get_leaves())
        )
    }

    /// Updates the node's counter of the height of the tree
    fn update_height(&mut self) {
        self.height = max(
            self.left.0.as_ref().map_or(0, |node| node.borrow().height),
            self.right.0.as_ref().map_or(0, |node| node.borrow().height)
        ) + 1;
    }

    /// Returns a reference to the [TreeBranch] that is pointed at the given path
    pub fn get_child(&self, pos: TreePath) -> Option<&TreeBranch<T, U>> {
        match pos {
            Left => self.left.0.as_ref(),
            Right => self.right.0.as_ref()
        }
    }

    /// Returns a reference to the [TreeBalance] owned by the node
    pub fn get_balance(&self) -> &U {
        &self.balance
    }

    /// Returns a mutable reference to the [TreeBalance] owned by the node
    pub fn get_balance_mut(&mut self) -> &mut U {
        &mut self.balance
    }

    /// Returns a reference to this node's parent [TreeBranch]
    pub fn get_parent(&self) -> Option<TreeBranch<T, U>> {
        Weak::upgrade(&self.parent)
    }

    /// Returns a mutable reference to the [Tree]
    /// used to point to the child along the given [TreePath]
    pub fn get_joint(&mut self, pos: TreePath) -> &mut Tree<T, U> {
        match pos {
            Left => &mut self.left,
            Right => &mut self.right
        }
    }

    /// Returns a mutable reference to the [Tree]
    /// used to point to the parent of this node
    pub fn get_parent_joint(&mut self) -> &mut TreeTrunk<T, U> {
        &mut self.parent
    }

    /// Removes the child [Tree] at the given path and returns it
    pub fn prune(&mut self, pos: TreePath) -> Tree<T, U> {
        // Perform replacement
        let pruned = match pos {
            Left => std::mem::replace(&mut self.left, Tree::new()),
            Right => std::mem::replace(&mut self.right, Tree::new())
        };

        // Remove parent reference from the path we just detached
        if let Some(ref p) = pruned.0 {
            *p.borrow_mut().get_parent_joint() = Weak::new()
        }

        pruned
    }

    /// Returns the path to take to search for the given key
    /// 
    /// Returns None if the key matches this node's key
    pub fn search(&self, key: &T) -> Option<TreePath> {
        match key.cmp(&self.key) {
            Ordering::Equal => None,
            Ordering::Greater => Some(Right),
            Ordering::Less => Some(Left)
        }
    }

    /// Returns the height of the tree rooted by this node
    pub fn get_height(&self) -> usize {
        self.height
    }

    /// Returns the number of leaves of the tree rooted by this node
    pub fn get_leaves(&self) -> usize {
        self.leaves
    }

    /// Finds which child path the provided [TreeNode] should be placed on
    /// relative to this node being used as the parent
    /// 
    /// Performs self.search(&child.key) but with the assumption that
    /// the keys of the two nodes do not match.
    /// The provided node does not need to be an actual child node.
    /// 
    /// # Panics
    /// 
    /// The function panics if the key of the child node and this node match
    /// ```rust,should_panic
    /// # use project2::tree::TreeNode;
    ///
    /// TreeNode::new_with(2).find_placement(&TreeNode::new_with(2))
    /// ```
    pub fn find_placement(&self, child: &TreeNode<T, U>) -> TreePath {
        self.search(&child.key).unwrap()
    }

    /// Returns the key and balance for this node consuming self
    pub fn pop(self) -> (T, U) {
        (self.key, self.balance)
    }

    /// Replaces the given key for this node returning the old key
    /// 
    /// # Panics
    /// 
    /// This function panics if the new key doesn't satisfy
    /// the condition: left child's key < new key < right child's key
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