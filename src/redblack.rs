use std::fmt::Debug;

use crate::tree::*;
use crate::inspect::*;

use RBColor::*;
/// Node colors
#[derive(Clone, Copy, PartialEq, Debug)]
enum RBColor {
    Red,
    Black
}

pub struct RedBlackBalance(RBColor);

/// Implementation of a balance for a binary tree
/// that effectively converts a [Tree] into into an Red Black Tree
impl TreeBalance for RedBlackBalance {
    fn rebalance_insert<T: Ord>(mut node: NodeInspector<T, Self>, path: (TreePath, TreePath)) -> TreePosition<T, Self> {

        // Get the parent and uncle colors
        let (pcolor, ucolor) = {
            (
                node.inspect_child(path.0).unwrap().inspect_balance(|b| b.0),
                node.inspect_child(path.0.reflect()).map_or(Black, |n| n.inspect_balance(|b| b.0))
            )
        };

        match (pcolor, ucolor) {
            // Perform recoloring
            (Red, Red) => {
                { 
                    node.inspect_child(TreePath::Right).unwrap().update_balance(|b| b.0 = Black);
                }
                {
                    node.inspect_child(TreePath::Left).unwrap().update_balance(|b| b.0 = Black); }
                {
                    if !node.inspect_is_root() {
                        node.update_balance(|b| b.0 = Red);
                    }
                }
            },
            // Perform rotation
            (Red, Black) => {
                node = node.rotate(path);
                { node.inspect_child(path.0.reflect()).unwrap().update_balance(|b| b.0 = Red);}
                { node.update_balance(|b| b.0 = Black); }
            },
            // Do nothing
            (Black, _) => ()
        }

        // We shouldn't need to continue to check for a
        // rebalance unless we performed a recolor only...
        // todo: verify if the tree works if we return root for all cases except recolor
        node.into_position(NodeOffset::Parent)

    }

    fn new() -> Self {
        RedBlackBalance (Red)
    }

    fn new_root() -> Self {
        RedBlackBalance (Black)
    }

    fn rebalance_delete<T: Ord>(mut node: NodeInspector<T, Self>, xpath: TreePath, popped_balance: &Self) -> TreePosition<T, Self> {
        // Check if the node removed was black, if not we don't need to investigate further
        if popped_balance.0 == Red {
            return node.into_position(NodeOffset::Root);
        }
        let spath = xpath.reflect(); // sibling path

        // Get the color of the node in the direction of the deletion,
        // the color of the sibling node, and (if any) the position of the sibling's red child
        let (xcolor, scolor, vpath) = {
            let snode = node.inspect_child(spath);
            let scolor = snode.as_ref().map_or(Black, |n| n.inspect_balance(|b| b.0));
            let vpath = snode.map(|snode| {
                let inline_color = snode.inspect_child(spath).map_or(Black, |n| n.inspect_balance(|b| b.0));
                let elbow_color = snode.inspect_child(xpath).map_or(Black, |n| n.inspect_balance(|b| b.0));
                // Prioritize the outermost child as if both children are red
                // using the outermost child prevents an extra rotation
                if inline_color == Red {
                    Some(spath)
                } else if elbow_color == Red {
                    Some(xpath)
                } else {
                    None
                }
            }).flatten();
            (
                node.inspect_child(xpath).map_or(Black, |n| n.inspect_balance(|b| b.0)),
                scolor,
                vpath
            )
        };

        // If the node in the direction of the deletion is black
        // that means we have a double black case
        if xcolor == Black {
            // If the sibling is also black we 
            if scolor == Black {
                // Get the parent's color (for later use)
                let pcolor = node.inspect_balance(|b| b.0);
                if let Some(path) = vpath {
                    // Perform rotation
                    node = node.rotate((spath, path));
                    node.inspect_child(TreePath::Right).unwrap().update_balance(|b| b.0 = Black);
                    node.inspect_child(TreePath::Left).unwrap().update_balance(|b| b.0 = Black);
                    node.update_balance(|b| b.0 = pcolor);
                    node.into_position(NodeOffset::Root)
                } else {
                    // Recolor, checking if we have another double black
                    node.inspect_child(spath).map(|mut n| n.update_balance(|b| b.0 = Red));
                    match pcolor {
                        Black => node.into_position(NodeOffset::Parent),
                        Red => {
                            node.update_balance(|b| b.0 = Black);
                            node.into_position(NodeOffset::Root)
                        }
                    }
                }
            } else {
                // Perform rotation and inspect the new parent of the double black child
                node = node.rotate((spath, spath));
                { node.inspect_child(xpath).unwrap().update_balance(|b| b.0 = Red);}
                { node.update_balance(|b| b.0 = Black); }
                node.into_position(NodeOffset::Child(xpath))
            }
        } else {
            // Nothing to do, return to root
            node.into_position(NodeOffset::Root)
        }
        
    }
}

impl Debug for RedBlackBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.0)
    }
}