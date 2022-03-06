use std::cmp::{Ordering, max};
use super::*;

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

    pub fn mark_root(&mut self) {
        self.balance.adjust_root();
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