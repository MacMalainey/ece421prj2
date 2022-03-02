use std::fmt::Display;
use std::fmt::Debug;
use std::cell::RefCell;
use std::rc::Rc;
use std::cmp::{Ordering, max};

use super::tree::TreeDir;
use super::tree::TreeDir::*;

type AVLBranch<T> = Rc<RefCell<AVLNode<T>>>;
type AVLTree<T> = Option<AVLBranch<T>>;

struct AVLNode<T: Ord + Debug> {
    key: T,
    height: usize,
    left: AVLTree<T>,
    right: AVLTree<T>,
}

enum TreeOpPath {
    Insert(TreeDir, TreeDir),
    Delete(TreeDir)
}

fn bst_rotate<T: Ord + Debug>(p: &AVLBranch<T>, direction: TreeDir) {
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

fn avl_check<T: Ord + Debug>(parent: &AVLBranch<T>, uncle: &AVLTree<T>) -> bool {
    let uheight = uncle.as_ref().map_or(0, |node| node.borrow().height);
    let pheight = parent.borrow().get_height();

    // If we properly insert every time this should never assert
    assert!(pheight >= uheight);
    pheight - uheight > 1
}

fn avl_check_beta<T: Ord + Debug>(parent: &AVLTree<T>, uncle: &AVLTree<T>) -> bool {
    let uheight = uncle.as_ref().map_or(0, |node| node.borrow().get_height());
    let pheight = parent.as_ref().map_or(0, |node| node.borrow().get_height());

    pheight - uheight > 1
}

fn avl_balance<T: Ord + Debug>(rnode: &AVLBranch<T>, op: TreeOpPath) {
    match op {
        TreeOpPath::Insert(ppath, xpath) => {

        },
        TreeOpPath::Delete(upath) => {
            let ppath = upath.reflect();
            let rebalance = {
                avl_check_beta(rnode.borrow().get_child(ppath), rnode.borrow().get_child(upath))
            };
            if rebalance {
                {
                    let r = rnode.borrow();
                    let pnode = r.get_child(ppath).as_ref().unwrap();
                    let xpath = {
                        let p = pnode.borrow();
                        p.get_child(Left)
                            .as_ref()
                            .map_or(
                                Right,
                                |xlnode| {
                                    let xr_height = p.get_child(Right).as_ref().map_or(0, |xrnode| xrnode.borrow().get_height());
                                    if xr_height > xlnode.borrow().get_height() {
                                        Right
                                    } else {
                                        Left
                                    }
                                }
                            )
                    };

                    if xpath != ppath {
                        bst_rotate(&pnode, xpath);
                    }
                }
                bst_rotate(rnode, ppath);
            }
        }
    }
}

fn bst_pop<T: Ord + Debug>(tree: &mut AVLTree<T>) -> Result<T, ()> {
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
    Ok(Rc::try_unwrap(popped.unwrap()).unwrap().into_inner().pop())
}

fn bst_delete<T: Ord + Debug>(root: &mut AVLTree<T>, key: &T) -> Option<T> {

    let (mut node_stack, mut tree_nav) = if let Some(root_node) = root {
        (
            Vec::with_capacity(root_node.borrow().get_height()),
            root_node.borrow().search(|nkey| key.cmp(nkey)).map(
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
                        .search(|nkey| key.cmp(nkey)) {
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

        avl_balance(&node, TreeOpPath::Delete(ppath));
    }

    Some(owned_key)
}

fn bst_insert<T: Ord + Debug>(rnode: &AVLBranch<T>, key: T) -> Option<TreeDir> {
    // If we didn't find the key grab the next child
    // otherwise we return
    let (parent_path, parent, uncle) = {
        let r = rnode.borrow();
        let next = r.search(|nkey| key.cmp(nkey));

        if let Some(path) = next {
            (
                path,
                // To prevent Rust from getting upset we temporarily get ownership the data here
                r.get_child(path).as_ref().map(|parent| Rc::clone(parent)),
                r.get_child(path.reflect()).as_ref().map(|uncle| Rc::clone(uncle)),  
            )
        } else {
            return None;
        }
    };

    // Further search the tree if possible otherwise we insert
    if let Some(pnode) = parent {
        match bst_insert(&pnode, key) {
            Some(child_path) => {
                // Keep this in it's own scope to prevent a panic
                { pnode.borrow_mut().update_height(); }

                if avl_check(&pnode, &uncle) {
                    if child_path != parent_path {
                        bst_rotate(&pnode, child_path);
                    }
                    bst_rotate(rnode, parent_path)
                }
            }
            None => return None
        };
    } else {
        let mut r = rnode.borrow_mut();
        *r.get_child_mut(parent_path) = Some(
            Rc::new(
                RefCell::new(
                    AVLNode::new_with(key)
                )
            )
        );
        r.update_height();
    }

    Some(parent_path)
}

impl <T: Ord + Debug> AVLNode<T> {

    fn new_with(key: T) -> Self {
        AVLNode {
            key,
            height: 1,
            left: None,
            right: None
        }
    }

    fn update_height(&mut self) {
        self.height = max(
            self.left.as_ref().map_or(0, |node| node.borrow().height),
            self.right.as_ref().map_or(0, |node| node.borrow().height)
        ) + 1;
    }

    fn get_child(&self, pos: TreeDir) -> &AVLTree<T> {
        match pos {
            Left => &self.left,
            Right => &self.right
        }
    }

    fn get_child_mut(&mut self, pos: TreeDir) -> &mut AVLTree<T> {
        match pos {
            Left => &mut self.left,
            Right => &mut self.right
        }
    }

    fn prune(&mut self, pos: TreeDir) -> AVLTree<T> {
        match pos {
            Left => self.left.take(),
            Right => self.right.take()
        }
    }

    fn search<F>(&self, compare: F) -> Option<TreeDir> 
        where F: FnOnce(&T) -> Ordering {
        match compare(&self.key) {
            Ordering::Equal => None,
            Ordering::Greater => Some(Right),
            Ordering::Less => Some(Left)
        }
    }

    fn get_height(&self) -> usize {
        self.height
    }

    fn pop(self) -> T {
        self.key
    }

    fn replace_key(&mut self, mut key: T) -> T {
        if let Some(ref node) = self.left {
            let n = node.borrow();
            assert_ne!(key.cmp(&n.key), Ordering::Less);
        }

        if let Some(ref node) = self.right {
            let n = node.borrow();
            assert_ne!(key.cmp(&n.key), Ordering::Greater);
        }

        std::mem::swap(&mut self.key, &mut key);

        key
    }

}

pub struct Tree<T: Ord + Debug>(AVLTree<T>);

impl <T: Ord + Debug> Tree<T> {
    pub fn new() -> Self {
        Tree(None)
    }

    pub fn insert(&mut self, key: T) {
        if let Some(ref branch) = self.0 {
            bst_insert(branch, key);
        } else {
            self.0 = Some(
                Rc::new(
                    RefCell::new(
                        AVLNode::new_with(key)
                    )
                )
            );
        }
    }

    pub fn delete(&mut self, key: &T) -> Option<T> {
        bst_delete(&mut self.0, key)
    }

    pub fn height(&self) -> usize {
        self.0.as_ref().map_or(0, |node| node.borrow().height)
    }
}

impl <T: Ord + Display + Debug> Display for Tree<T> {

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

impl <T: Ord + Display + Debug> Display for AVLNode<T> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self.left {
            Some(ref node) => std::fmt::Display::fmt(&node.borrow(), f),
            None => Ok(())
        }.and_then(|_| write!(f, "{}, ", self.key)).and_then(|_|
            match self.right {
                Some(ref node) => std::fmt::Display::fmt(&node.borrow(), f),
                None => Ok(())
            }
        )
    }
}

impl <T: Ord + Debug> Debug for AVLNode<T> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let mut builder = f.debug_struct(&format!("{:?}", &self.key));
        builder.field("height", &self.height);
        match self.right {
            Some(ref node) => builder.field("right", &node.borrow()),
            None => builder.field("right", &"None")
        };
        match self.left {
            Some(ref node) => builder.field("left", &node.borrow()),
            None => builder.field("left", &"None")
        };
        builder.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::Tree;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn manual_check_order() {
        let nums = [
            40, 65, 55,
            57, 58, 75, 60, 59
        ];
        let mut tree: Tree<u32> = Tree::new();

        for num in &nums {
            println!("----- Insert: {} -------", num);
            tree.insert(*num);
            println!("{:#?}", tree);
        }
    }

    #[test]
    fn manual_check_delete() {
        let nums = [
            40, 65, 55,
            57, 58, 75, 60, 59
        ];
        let mut tree: Tree<u32> = Tree::new();

        for num in &nums {
            tree.insert(*num);
        }
        println!("----- Insert: {} -------", 40);
        assert_eq!(tree.delete(&58), Some(58));
        println!("{:#?}", tree);
    }
}