use crate::tree::*;
use crate::tree::TreePath::*;
use crate::inspect::*;
use std::rc::Rc;
use std::cell::RefCell;

pub fn bst_insert<T: Ord, U: TreeBalance>(root: Tree<T, U>, key: T) -> Tree<T, U> {
    if let Some(mut p) = root.branch().map(|r| Rc::clone(r)) {
        // Find the parent node to insert to and the path to insert on...
        // Or if the tree is empty we insert at root and be done
        let mut xpath;
        loop {
            let child = {
                let pnode = p.borrow();
                let search_path = pnode.search(&key);
                if let Some(path) = search_path {
                    xpath = path;
                } else {
                    return root
                }

                pnode.get_child(xpath).map(|node| Rc::clone(node))
            };

            if let Some(x) = child {
                p = x;
            } else {
                break;
            }
        }

        // Perform insert
        let grandparent = {
            let mut pnode = p.borrow_mut();
            *pnode.get_joint(xpath) = Tree::new_with(
                Rc::new(
                    RefCell::new(
                        TreeNode::new_with_parent(key, Rc::downgrade(&p))
                    )
                )
            );
            pnode.update();
            pnode.get_parent().map(|b| {
                let placement = b.borrow().find_placement(&pnode);
                (b, placement)
            })
        };

        // Rebalance Tree
        if let Some((mut r, mut ppath)) = grandparent {
            loop {
                let (current, next_pos) = U::rebalance_insert(NodeInspector::from(r), (ppath, xpath)).into_inner();
                r = current;
                let next = {
                    let mut rnode = r.borrow_mut();
                    rnode.update();
                    match next_pos {
                        NodeOffset::Root => break,
                        NodeOffset::Parent =>
                            rnode.get_parent().map(|b| {
                                let placement = b.borrow().find_placement(&rnode);
                                (b, placement)
                        }),
                        NodeOffset::Child(_) => panic!("Should not happen!")
                    }
                };

                if let Some((n, path)) = next {
                    xpath = ppath;
                    ppath = path;
                    r = n;
                } else {
                    break;
                }
            }

            // If we have anything more to go up the tree, do now
            let mut next = { r.borrow().get_parent() };
            while let Some(n) = next {
                r = n;
                let mut rnode = r.borrow_mut();
                rnode.update();
                next = rnode.get_parent();
            }

            Tree::new_with(r)
        } else {
            Tree::new_with(p)
        }

    } else {
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
enum DeletePosition<T: Ord, U: TreeBalance> {
    Child(TreeBranch<T, U>, TreePath),
    Root
}

pub fn bst_delete<T: Ord, U: TreeBalance>(root: Tree<T, U>, key: &T) -> (Tree<T, U>, Option<T>) {
    if let Some(mut p) = root.into_inner() {
        // Find the parent node of the node we wish to delete
        // Or if the tree is empty we just return root
        let mut root_keep_alive = Tree::new_with(Rc::clone(&p));
        let mut xpath = {
            p.borrow().search(key)
        };
        if let Some(mut path) = xpath {
            loop {
                let next = {
                    let pnode = p.borrow();
                    pnode.get_child(path).map(|node| {
                        (Rc::clone(node), node.borrow().search(key))
                    })
                };

                match next {
                    Some((_, None)) => {
                        xpath = Some(path);
                        break
                    },
                    Some((x, Some(next_path))) => {
                        path = next_path;
                        p = x;
                    }
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

                    // If yes, pop the b.borrow().find_placement(&pnode)node. Otherwise continue travelling the tree
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
            let (current, next_pos) = U::rebalance_delete(NodeInspector::from(p), xpath, &mut balance).into_inner();
            // Based on the instructions from the balancer we either rebalance a child or the root next
            
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

        (Tree::new_with(p), Some(key))
    } else {
        (Tree::new(), None)
    }
}

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

pub fn bst_search<T: Ord, U: TreeBalance>(root: &Tree<T, U>, key: &T) -> bool {
    let mut next = root.branch().map(|b| Rc::clone(b));
    while let Some(n) = next {
        let node = n.borrow();
        next = match node.search(key) {
            None => return true,
            Some(path) => node.get_child(path).map(|b| Rc::clone(b))
        };
    }

    false
}