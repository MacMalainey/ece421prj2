use std::fmt::Display;
use std::fmt::Debug;
use std::cell::RefCell;
use std::rc::Rc;
use super::avl::*;

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

pub trait TreeNode<T: Ord> {
    fn new_with(key: T) -> Self;

    fn update_height(&mut self);
    fn get_height(&self) -> usize;

    fn get_child(&self, pos: TreeDir) -> &AVLTree<T>;
    fn get_child_mut(&mut self, pos: TreeDir) -> &mut AVLTree<T>;
    fn prune(&mut self, pos: TreeDir) -> AVLTree<T>;

    fn search(&self, key: &T) -> Option<TreeDir>;

    fn pop(self) -> T;
    
    fn insert(&mut self, pos: TreeDir);
    fn replace_key(&mut self, key: T) -> T;
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

pub fn bst_rotate<T: Ord>(p: &AVLBranch<T>, direction: TreeDir) {
    // Break apart the tree
    let x = {
        p.borrow_mut().prune(direction).unwrap()
    };
    let v = {
        x.borrow_mut().prune(direction.reflect())
    };

    // Swap P and X
    p.swap(&x);

    // Add V to P
    { *x.borrow_mut().get_child_mut(direction) = v };

    // Add P to X
    { *p.borrow_mut().get_child_mut(direction.reflect()) = Some(Rc::clone(&x)) };

    // Update heights
    { x.borrow_mut().update_height() };
    { p.borrow_mut().update_height() };
}

pub fn bst_pop<T: Ord>(tree: &mut AVLTree<T>) -> Result<T, ()> {
    let successor = {
        let mut x = tree.as_ref().unwrap().borrow_mut();
        if x.get_child(Right).is_some() && x.get_child(Left).is_some() {
            return Err(())
        } else {
            x.prune(Right)
                .or_else(|| x.prune(Left))
        }
    };

    let popped = std::mem::replace(tree, successor);
    Ok(
        Rc::try_unwrap(
            popped.unwrap()
        ).map_err(|_| ()) // map_err() call to satisfy the Debug trait bound
        .unwrap().into_inner().pop()
    )
}

pub fn bst_delete<T: Ord>(root: &mut AVLTree<T>, key: &T) -> Option<T> {

    let (mut node_stack, mut tree_nav) = if let Some(root_node) = root {
        (
            Vec::with_capacity(root_node.borrow().get_height()),
            root_node.borrow().search(&key).map(
                |path| {
                    (path, Rc::clone(root_node))
                }
            )
        )
    } else { 
        return None
    };

    // Find the node we want to remove
    tree_nav = if let Some((mut xpath, mut pnode)) = tree_nav {
        loop {
            let (path, node) = {
                let p = pnode.borrow();
                let xtree = p.get_child(xpath);

                if let Some(xnode) = xtree {
                    if let Some(path) = xnode
                        .borrow()
                        .search(&key) {
                            (path, Rc::clone(&xnode))
                    } else {
                        break; 
                    }
                } else {
                    return None;
                }
            };

            node_stack.push((xpath, Rc::clone(&pnode)));
            xpath = path;
            pnode = node;
        }

        Some((xpath, pnode))
    } else {
        tree_nav
    };

    let (owned_key, pnode) = {
        let pnode;
        let mut p;
        let (xtree, pnode_clone) = if let Some((path, node)) = tree_nav {
            pnode = node;
            let clone = Rc::clone(&pnode);
            p = pnode.borrow_mut();
            (p.get_child_mut(path), Some(clone))
        } else {
            (root, None)
        };

        // We found the node to delete, now we check if we need to perform a swap
        if let Ok(key) = bst_pop(xtree) {
            // No swap needed
            (key, pnode_clone.unwrap())
        } else {
            // We need to swap with the bottom-left most node
            let mut xpath = Right;
            let next_path = Left;

            let mut pnode = Rc::clone(xtree.as_ref().unwrap());
            {
                let mut swap_node = Rc::clone(pnode.borrow().get_child(xpath).as_ref().unwrap());
                loop {
                    let node = {
                        let s = swap_node.borrow();
                        let next_tree = s.get_child(
                            next_path
                        );
            
                        if let Some(next_node) = next_tree {
                            Rc::clone(next_node)
                        } else {
                            break;
                        }
                    };
    
                    node_stack.push((xpath, pnode));
                    pnode = swap_node;
                    swap_node = node;
    
                    xpath = Left;
                }
            }

            // Get key and perform swap
            let swap_key =
                bst_pop(
                    pnode.borrow_mut()
                        .get_child_mut(xpath)
                ).unwrap();

            (xtree.as_ref().unwrap().borrow_mut().replace_key(swap_key), pnode)
        }
    };

    pnode.borrow_mut().update_height();

    while let Some((ppath, node)) = node_stack.pop() {
        { node.borrow_mut().update_height(); }

        avl_balance(&node, TreeOp::Delete(ppath));
    }

    Some(owned_key)
}

pub fn bst_insert<T: Ord>(root: &mut AVLTree<T>, key: T) {
    let (mut node_stack, (mut xpath, mut pnode)) = if let Some(root_node) = root {
        (
            Vec::with_capacity(root_node.borrow().get_height()),
            if let Some(path) = root_node.borrow().search(&key) {
                (path, Rc::clone(root_node))
            } else {
                return
            }
        )
    } else { 
        *root = Some(
            Rc::new(
                RefCell::new(
                    AVLNode::new_with(key)
                )
            )
        );
        return
    };

    // Find where we want to insert
    loop {
        let (path, node) = {
            let mut p = pnode.borrow_mut();
            let xtree = p.get_child_mut(xpath);

            if let Some(xnode) = xtree {
                if let Some(path) = xnode
                    .borrow()
                    .search(&key) {
                        (path, Rc::clone(&xnode))
                } else {
                    return;
                }
            } else {
                *xtree = Some(
                    Rc::new(
                        RefCell::new(
                            AVLNode::new_with(key)
                        )
                    )
                );
                p.update_height();
                break;
            }
        };

        node_stack.push((xpath, Rc::clone(&pnode)));
        xpath = path;
        pnode = node;
    }

    while let Some((ppath, node)) = node_stack.pop() {
        { node.borrow_mut().update_height(); }

        avl_balance(&node, TreeOp::Insert(ppath, xpath));

        xpath = ppath;
    }
}

pub struct Tree<T: Ord>(AVLTree<T>);