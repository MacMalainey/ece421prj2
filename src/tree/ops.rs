use super::*;

pub fn bst_full_rotate<T: Ord, U>(rnode: &TreeBranch<T, U>, ppath: TreeDir, xpath: TreeDir)
    where U: TreeBalance<T>
{
    if xpath != ppath {
        let r = rnode.borrow();
        let pnode = r.get_child(ppath).as_ref().unwrap();
        bst_rotate(pnode, xpath);
    }

    bst_rotate(rnode, ppath);
}

pub fn bst_rotate<T: Ord, U>(p: &TreeBranch<T, U>, direction: TreeDir)
    where U: TreeBalance<T>
{
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

pub fn bst_pop<T: Ord, U>(tree: &mut Option<TreeBranch<T, U>>) -> Result<T, ()>
    where U: TreeBalance<T>
{
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
        Rc::try_unwrap(popped.unwrap()).map_err(|_| ()) // map_err() call to satisfy the Debug trait bound
        .unwrap().into_inner().pop()  

    )
}

pub fn bst_delete<T: Ord, U>(root: &mut Option<TreeBranch<T, U>>, key: &T) -> Option<T>
    where U: TreeBalance<T>
{

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
                            (path, Rc::clone(xnode))
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

    // Perform node pop and return key
    let owned_key = {
        let pnode;
        let mut p;
        let xtree = if let Some((path, node)) = tree_nav {
            node_stack.push((path, Rc::clone(&node)));

            pnode = node;
            p = pnode.borrow_mut();
            p.get_child_mut(path)
        } else {
            root
        };

        // We found the node to delete, now we check if we need to perform a swap
        if let Ok(key) = bst_pop(xtree) {
            // No swap needed
            key
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

                        node_stack.push((xpath, Rc::clone(&pnode)));
                        if let Some(next_node) = next_tree {
                            Rc::clone(next_node)
                        } else {
                            break;
                        }
                    };
                    
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

            xtree.as_ref().unwrap().borrow_mut().replace_key(swap_key)
        }
    };

    while let Some((ppath, node)) = node_stack.pop() {
        let balance_path = {
            let mut n = node.borrow_mut();
            n.update_height();
            n.rebalance(TreeOp::Delete(ppath))
        };

        if let Some((ppath, xpath)) = balance_path {
            bst_full_rotate(&node, ppath, xpath)
        }
    }

    Some(owned_key)
}

pub fn bst_insert<T: Ord, U>(root: &mut Option<TreeBranch<T, U>>, key: T)
    where U: TreeBalance<T>
{
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
        let root_node = TreeNode::new_with(key);
        root_node.mark_root();
        *root = Some(
            Rc::new(
                RefCell::new(
                    root_node
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
                        (path, Rc::clone(xnode))
                } else {
                    return;
                }
            } else {
                *xtree = Some(
                    Rc::new(
                        RefCell::new(
                            TreeNode::new_with(key)
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
        let balance_path = {
            let mut n = node.borrow_mut();
            n.update_height();
            n.rebalance(TreeOp::Insert(ppath, xpath))
        };

        if let Some((ppath, xpath)) = balance_path {
            bst_full_rotate(&node, ppath, xpath)
        }

        xpath = ppath;
    }

    root.as_ref().unwrap().borrow().mark_root()
}