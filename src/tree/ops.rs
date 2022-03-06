use std::rc::Rc;
use std::cell::RefCell;

use super::*;
use super::TreePath::*;
use super::inspect::*;

/// Perform an insertion using a given key
/// on a binary tree with the given root
/// 
/// # Issues
/// 
/// Behaviour is undefined if root is actually the root of a subtree 
/// and not the full tree (need to add panic if root has a parent node)
pub fn bst_insert<T: Ord, U: TreeBalance>(root: Tree<T, U>, key: T) -> Tree<T, U> {
    // Unwrap the root
    if let Some(mut p) = root.branch().map(|r| Rc::clone(r)) {
        // Find the parent node to insert to and the path to insert on...
        // Or if the tree is empty we insert at root and be done
        let mut xpath;
        loop {
            // Get the next child node
            let child = {
                let pnode = p.borrow();
                let search_path = pnode.search(&key);
                // Get the next path to search down
                if let Some(path) = search_path {
                    xpath = path;
                } else {
                    // Found key in tree already, just return root
                    return root
                }

                pnode.get_child(xpath).map(|node| Rc::clone(node))
            };

            // If child exists on the found path
            // we set it as our parent and continue traversal
            if let Some(x) = child {
                p = x;
            } else {
                // Found an empty node, break
                break;
            }
        }

        // Perform insert and return a reference to the grandparent
        let grandparent = {
            let mut pnode = p.borrow_mut();
            // Update parent node
            *pnode.get_joint(xpath) = Tree::new_with(
                Rc::new(
                    RefCell::new(
                        TreeNode::new_with_parent(key, Rc::downgrade(&p))
                    )
                )
            );
            pnode.update();
            // Get grandparent info for rebalancing
            pnode.get_parent().map(|b| {
                let placement = b.borrow().find_placement(&pnode);
                (b, placement)
            })
        };

        // Rebalance Tree
        if let Some((mut r, mut ppath)) = grandparent {
            loop {
                // Perform the rebalance
                let (current, next_pos) = U::rebalance_insert(NodeInspector::open(r), (ppath, xpath)).into_inner();
                r = current;
                // Get the next node based off next_pos
                let next = {
                    let mut rnode = r.borrow_mut();
                    rnode.update();
                    match next_pos {
                        NodeOffset::Root => break, // Gets returned if we no-longer need to rebalance
                        NodeOffset::Parent =>
                            rnode.get_parent().map(|b| {
                                let placement = b.borrow().find_placement(&rnode);
                                (b, placement)
                        }),
                        // In our implemented cases (AVL, Red Black, Unbalanced) this shouldn't happen.
                        // However, to allow for implementing other balancing methods it might be useful to allow this case
                        // If more time was budgetted this might be worth implementing
                        NodeOffset::Child(_) => panic!("Should not happen!")
                    }
                };

                // Check if we have another path to traverse
                // and update locals accordingly otherwise break
                if let Some((n, path)) = next {
                    xpath = ppath;
                    ppath = path;
                    r = n;
                } else {
                    break;
                }
            }

            // If we have anything more to go up the tree, do now
            // updating each node's understanding of the tree as we do
            let mut next = { r.borrow().get_parent() };
            while let Some(n) = next {
                r = n;
                let mut rnode = r.borrow_mut();
                rnode.update();
                next = rnode.get_parent();
            }

            // Return a tree wrapping root
            Tree::new_with(r)
        } else {
            // Return a tree wrapping the parent
            // since there was no grandparent
            Tree::new_with(p)
        }

    } else {
        // Tree is empty, return fresh new node
        Tree::new_with(
            Rc::new(
                RefCell::new(
                    TreeNode::new_with(key)
                )
            )
        )
    }
}

// Helper enum, only used for bst_delete
// Could be replaced with Option type after refactor
enum DeletePosition<T: Ord, U: TreeBalance> {
    // Deleted a child node
    Child(TreeBranch<T, U>, TreePath),
    // Deleted the root node
    Root
}

/// Perform a deletion using a given key on a binary
/// tree with the given root and return the key
/// 
/// # Issues
/// 
/// Behaviour is undefined if root is actually the root of a subtree 
/// and not the full tree (should add panic if root has a parent node)
pub fn bst_delete<T: Ord, U: TreeBalance>(root: Tree<T, U>, key: &T) -> (Tree<T, U>, Option<T>) {
    // Find the parent node of the node we wish to delete
    // Or if the tree is empty we just return root
    if let Some(mut p) = root.into_inner() {
        // DO NOT LET THIS VARIABLE DIE OTHERWISE THE TREE WILL BEGIN TO DEALLOCATE
        let mut root_keep_alive = Tree::new_with(Rc::clone(&p));
        let mut xpath = {
            p.borrow().search(key)
        };
        // If we aren't deleting the root we need to find the
        // parent node and the path to the child node to delete
        if let Some(mut path) = xpath {
        // Could probably be refactored to a while let
            loop {
                // Get the next parent and the next path
                let next = {
                    let pnode = p.borrow();
                    pnode.get_child(path).map(|node| {
                        (Rc::clone(node), node.borrow().search(key))
                    })
                };

                match next {
                    // If we have a parent but no path, that means we found the node
                    Some((_, None)) => {
                        xpath = Some(path);
                        break
                    },
                    // If we have another path to go down keep up traversal
                    Some((x, Some(next_path))) => {
                        path = next_path;
                        p = x;
                    }
                    // We hit the bottom of the tree, return root and no key
                    None => return (root_keep_alive, None)
                }
            }
        }

        // Perform delete
        let (key, mut balance, mut p, mut xpath) = {
            // Attempt the pop on x, re-write parent
            let (popped, position) = if let Some(path) = xpath {
                // Pop relative to the parent
                let popped = bst_pop(p.borrow_mut().get_joint(path));
                (popped, DeletePosition::Child(p, path))
            } else {
                // If there is not xpath that means the node we wish to pop is root
                drop(root_keep_alive); // Drop here to remove extra Rc pointer that is needed so that the Weak references don't die
                root_keep_alive = Tree::new_with(p);
                (bst_pop(&mut root_keep_alive), DeletePosition::Root)
            };

            // If the pop was successful, return the result...
            // Otherwise find a node to swap with
            if let Some((key, balance)) = popped {
                match position {
                    DeletePosition::Child(p, path) => (key, balance, p, path),
                    DeletePosition::Root => return (root_keep_alive, Some(key))
                }
            } else {
                // Store the node we will swap with (original x)
                let to_swap = match position {
                    DeletePosition::Child(p, path) => Rc::clone(p.borrow().get_child(path).unwrap()),
                    DeletePosition::Root => {
                        Rc::clone(root_keep_alive.branch().unwrap())
                    }
                };

                let mut xpath = Right;
                let mut p = Rc::clone(&to_swap);
                let key;
                let balance;
                loop {
                    // Check if we should pop
                    let pop = {
                        let pnode = p.borrow();
                        let xnode = pnode.get_child(xpath).unwrap().borrow();
                        xnode.get_child(Left).is_none()
                    };

                    // If yes, pop the child node at xpath. Otherwise continue travelling the tree
                    if pop {
                        let (popped_key, popped_balance) = {
                            let mut pnode = p.borrow_mut();
                            bst_pop(pnode.get_joint(xpath)).unwrap()
                        };
                        balance = popped_balance;
                        key = to_swap.borrow_mut().replace_key(popped_key);
                        break;
                    } else {
                        let next_p = {
                            Rc::clone(&p.borrow().get_child(xpath).unwrap())
                        };
                        xpath = Left;
                        p = next_p;
                    }
                }
                
                (key, balance, p, xpath)
            }
        };

        // Rebalance Tree
        loop {
            let (current, next_pos) = U::rebalance_delete(NodeInspector::open(p), xpath, &mut balance).into_inner();
    
            // Based on the instructions from the balancer we either rebalance a child, parent, or stop and just go to root
            let next = {
                p = current;
                let mut pnode = p.borrow_mut();
                pnode.update();
                match next_pos {
                    NodeOffset::Root => break,
                    NodeOffset::Parent => {
                        pnode.get_parent().map(|b| {
                            let placement = b.borrow().find_placement(&pnode);
                            (b, placement)
                        })
                    },
                    NodeOffset::Child(path) => {
                        let new = Rc::clone(pnode.get_child(path).unwrap());
                        Some((new, path))
                    }
                }
            };

            // If there is more to traverse, do it
            if let Some((n, path)) = next {
                xpath = path;
                p = n;
            } else {
                break;
            }
        }

        // If we have anything more to go up the tree, do now
        let mut next = { p.borrow().get_parent() };
        while let Some(n) = next {
            p = n;
            let mut rnode = p.borrow_mut();
            rnode.update();
            next = rnode.get_parent();
        }

        // Return the new root and the key
        (Tree::new_with(p), Some(key))
    } else {
        // Tree is empty return an empty tree and no key
        (Tree::new(), None)
    }
}

/// Removes a node at the given reference
/// 
/// Removes the node pointer stored at the passed in reference replacing
/// it with a child node or an empty node if no children.  This allows
/// for the removal to happen "in-place" and no parent needs to exist to perform.
/// Returns the key and balancer of the popped node.
/// 
/// If the node has 2 children we return None as we cannot pop a node with more than one child
/// 
/// # Panics
/// 
/// Due to the nature of the tree structure this will fail and panic if a strong
/// pointer to a the "node to pop" exists outside of this function.
/// 
/// This function will panic if the node to remove is empty
pub fn bst_pop<T: Ord, U: TreeBalance>(tree: &mut Tree<T, U>) -> Option<(T, U)> {
    let successor = {
        let mut x = tree.branch().unwrap().borrow_mut();
        if x.get_child(Right).is_some() && x.get_child(Left).is_some() {
            return None
        } else {
            // Detatch the successor
            let mut suc = x.prune(Right);
            if suc.is_empty() {
                suc = x.prune(Left)
            }

            // Change the successor's parent
            if let Some(s) = suc.branch() {
                let parent = x.get_parent().map_or(
                    std::rc::Weak::new(),
                    |b| Rc::downgrade(&b)
                );
                *s.borrow_mut().get_parent_joint() = parent;
            }
            suc
        }
    };

    let popped = std::mem::replace(tree, successor);
    Some(
        Rc::try_unwrap(popped.into_inner().unwrap()).map_err(|_| ()) // map_err() call to satisfy the Debug trait bound
        .unwrap().into_inner().pop()
    )
}

/// Performs a rotation on a tree around opposite to the given direction
/// 
/// After the rotation is complete it will return the new root node for the subtree
/// 
/// The rotation is mirrored (i.e. a direction of Left will rotate Right) as the direction is in
/// reference to where the new root node should come from (i.e. a Right rotation brings up the Left child)
/// This is to stay consistent with the [NodeInspector.rotate] function's definition
/// 
/// # Panics
/// 
/// This function panics if the child node in the given direction is empty
pub fn bst_rotate<T: Ord, U: TreeBalance>(p: TreeBranch<T, U>, direction: TreePath) -> TreeBranch<T, U> {
    // Get the grandparent
    let grandparent = {
        p.borrow().get_parent()
    };

    // Break apart the tree
    let x = {
        p.borrow_mut().prune(direction).into_inner().unwrap()
    };
    let grandchild = {
        x.borrow_mut().prune(direction.reflect())
    };

    // Connect the grandchild and old parent together
    {
        if let Some(v) = grandchild.branch() {
            *v.borrow_mut().get_parent_joint() = Rc::downgrade(&p);
        }
        let mut pnode = p.borrow_mut();
        *pnode.get_joint(direction) = grandchild;
        pnode.update();
    }

    // Connect the old child and old parent together
    { *x.borrow_mut().get_joint(direction.reflect()) = Tree::new_with(Rc::clone(&p)); }
    { *p.borrow_mut().get_parent_joint() = Rc::downgrade(&x); }

    // Connect the old child and grandparent together
    if let Some(r) = grandparent {
        let direction = {
            r.borrow().find_placement(&x.borrow())
        };
        { *r.borrow_mut().get_joint(direction) = Tree::new_with(Rc::clone(&x)) };
        { *x.borrow_mut().get_parent_joint() = Rc::downgrade(&r) };
    }

    x
}

/// Performs a binary search on a tree with the given root
/// 
/// Returns true if the node is found
pub fn bst_search<T: Ord, U: TreeBalance>(root: &Tree<T, U>, key: &T) -> bool {

    // Traverse the tree looking for the key
    let mut next = root.branch().map(|b| Rc::clone(b));
    while let Some(n) = next {
        let node = n.borrow();
        next = match node.search(key) {
            None => return true, // Key found
            Some(path) => node.get_child(path).map(|b| Rc::clone(b)) // Key looking
        };
    }

    // Hit the end of the tree, therefore it was not found
    false
}