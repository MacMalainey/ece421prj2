use super::*;

pub fn bst_full_rotate<T: Ord, U>(rnode: &U, ppath: TreeDir, xpath: TreeDir)
    where U: TreeBranch<T>
{
    if xpath != ppath {
        let r = rnode.borrow();
        let pnode = r.get_child(ppath).as_ref().unwrap();
        bst_rotate(pnode, xpath);
    }

    bst_rotate(rnode, ppath);
}

pub fn bst_rotate<T: Ord, U>(p: &U, direction: TreeDir)
    where U: TreeBranch<T>
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
    { *p.borrow_mut().get_child_mut(direction.reflect()) = Some(x.clone()) };

    // Update heights
    { x.borrow_mut().update_height() };
    { p.borrow_mut().update_height() };
}

pub fn bst_pop<T: Ord, U>(tree: &mut Option<U>) -> Result<T, ()>
    where U: TreeBranch<T>
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
        popped.unwrap().into_inner().map_err(|_| ()) // map_err() call to satisfy the Debug trait bound
        .unwrap().into_inner().pop() 
    )
}

pub fn bst_delete<T: Ord, U>(root: &mut Option<U>, key: &T) -> Option<T>
    where U: TreeBranch<T>
{

    let (mut node_stack, mut tree_nav) = if let Some(root_node) = root {
        (
            Vec::with_capacity(root_node.borrow().get_height()),
            root_node.borrow().search(&key).map(
                |path| {
                    (path, root_node.clone())
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
                            (path, xnode.clone())
                    } else {
                        break; 
                    }
                } else {
                    return None;
                }
            };

            node_stack.push((xpath, pnode.clone()));
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
            let clone = pnode.clone();
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

            let mut pnode = xtree.as_ref().unwrap().clone();
            {
                let mut swap_node = pnode.borrow().get_child(xpath).as_ref().unwrap().clone();
                loop {
                    let node = {
                        let s = swap_node.borrow();
                        let next_tree = s.get_child(
                            next_path
                        );
            
                        if let Some(next_node) = next_tree {
                            next_node.clone()
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

        if let Some((ppath, xpath)) =
            node.rebalance(TreeOp::Delete(ppath))
        {
            bst_full_rotate(&node, ppath, xpath)
        }
    }

    Some(owned_key)
}

pub fn bst_insert<T: Ord, U>(root: &mut Option<U>, key: T)
    where U: TreeBranch<T>
{
    let (mut node_stack, (mut xpath, mut pnode)) = if let Some(root_node) = root {
        (
            Vec::with_capacity(root_node.borrow().get_height()),
            if let Some(path) = root_node.borrow().search(&key) {
                (path, root_node.clone())
            } else {
                return
            }
        )
    } else { 
        *root = Some(
            U::new(
                TreeNode::new_with(key)
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
                        (path, xnode.clone())
                } else {
                    return;
                }
            } else {
                *xtree = Some(
                    U::new(TreeNode::new_with(key))
                );
                p.update_height();
                break;
            }
        };

        node_stack.push((xpath, pnode.clone()));
        xpath = path;
        pnode = node;
    }

    while let Some((ppath, node)) = node_stack.pop() {
        { node.borrow_mut().update_height(); }

        if let Some((ppath, xpath)) =
            node.rebalance(TreeOp::Insert(ppath, xpath))
        {
            bst_full_rotate(&node, ppath, xpath)
        }

        xpath = ppath;
    }
}