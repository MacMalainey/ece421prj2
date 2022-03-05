use std::fmt::Debug;

use crate::tree::*;
use crate::inspect::*;

use RBColor::*;
#[derive(Clone, Copy, PartialEq, Debug)]
enum RBColor {
    Red,
    Black
}

pub struct RedBlackBalance(RBColor);

impl TreeBalance for RedBlackBalance {
    fn rebalance_insert<T: Ord>(mut node: NodeInspector<T, Self>, path: (TreePath, TreePath)) -> NextTreePosition<T, Self> {

        let (pcolor, ucolor) = {
            (
                node.inspect_child(path.0).unwrap().inspect_balance(|b| b.0),
                node.inspect_child(path.0.reflect()).map_or(Black, |n| n.inspect_balance(|b| b.0))
            )
        };

        match (pcolor, ucolor) {
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
            (Red, Black) => {
                node = node.rotate(path);
                { node.inspect_child(path.0.reflect()).unwrap().update_balance(|b| b.0 = Red);}
                { node.update_balance(|b| b.0 = Black); }
            },
            (Black, _) => ()
        }

        node.into_next_position(TreePosition::Parent)

    }

    fn new() -> Self {
        RedBlackBalance (Red)
    }

    fn new_root() -> Self {
        RedBlackBalance (Black)
    }

    fn rebalance_delete<T: Ord>(mut node: NodeInspector<T, Self>, xpath: TreePath, popped_balance: &Self) -> NextTreePosition<T, Self> {
        if popped_balance.0 == Red {
            return node.into_next_position(TreePosition::Root);
        }
        let spath = xpath.reflect();
        let (xcolor, scolor, vpath) = {
            let snode = node.inspect_child(spath);
            let scolor = snode.as_ref().map_or(Black, |n| n.inspect_balance(|b| b.0));
            let vpath = snode.map(|snode| {
                let inline_color = snode.inspect_child(spath).map_or(Black, |n| n.inspect_balance(|b| b.0));
                let elbow_color = snode.inspect_child(xpath).map_or(Black, |n| n.inspect_balance(|b| b.0));
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

        if xcolor == Black {
            if scolor == Black {
                let pcolor = node.inspect_balance(|b| b.0);
                if let Some(path) = vpath {
                    node = node.rotate((spath, path));
                    node.inspect_child(TreePath::Right).unwrap().update_balance(|b| b.0 = Black);
                    node.inspect_child(TreePath::Left).unwrap().update_balance(|b| b.0 = Black);
                    node.update_balance(|b| b.0 = pcolor);
                    node.into_next_position(TreePosition::Root)
                } else {
                    node.inspect_child(spath).map(|mut n| n.update_balance(|b| b.0 = Red));
                    match pcolor {
                        Black => node.into_next_position(TreePosition::Parent),
                        Red => {
                            node.update_balance(|b| b.0 = Red);
                            node.into_next_position(TreePosition::Root)
                        }
                    }
                }
            } else {
                node = node.rotate((spath, spath));
                { node.inspect_child(xpath).unwrap().update_balance(|b| b.0 = Red);}
                { node.update_balance(|b| b.0 = Black); }
                node.into_next_position(TreePosition::Child(xpath))
            }
        } else {
            node.into_next_position(TreePosition::Root)
        }
        
    }
}

impl Debug for RedBlackBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.0)
    }
}