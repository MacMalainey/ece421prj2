pub mod avl;
pub mod tree;
pub mod ops;
pub mod inspect;
pub mod redblack;

use tree::Tree;
pub type AVLTree<T> = Tree<T, avl::AVLBalance>;
pub type RedBlackTree<T> = Tree<T, redblack::RedBlackBalance>;

#[cfg(test)]
mod tests {
    use super::{AVLTree, RedBlackTree};

    #[test]
    fn manual_avl_check_insert() {
        let nums = [
            40, 65, 55,
            57, 58, 75, 60, 59
        ];
        let mut tree: AVLTree<u32> = AVLTree::new();

        for num in &nums {
            println!("----- Insert: {} -------", num);
            tree.insert(*num);
            println!("{:#?}", tree);
        }
    }

    #[test]
    fn manual_avl_check_delete() {
        let nums = [
            40,
            65,
            55,
            57, 58, 75, 60, 59
        ];
        let mut tree: AVLTree<u32> = AVLTree::new();

        for num in &nums {
            tree.insert(*num);
        }
        assert_eq!(tree.delete(&55), Some(55));
        println!("{:#?}", tree);
    }

    #[test]
    fn manual_rb_check_insert() {
        let nums = [
            40, 65, 55,
            57, 58, 75, 60, 59
        ];
        let mut tree: RedBlackTree<u32> = RedBlackTree::new();

        for num in &nums {
            println!("----- Insert: {} -------", num);
            tree.insert(*num);
            println!("{:#?}", tree);
        }
    }

    // #[test]
    // fn manual_rb_check_delete() {
    //     let nums = [
    //         40, 65, 55,
    //         57, 58, 75, 60, 59
    //     ];
    //     let mut tree: RedBlackTree<u32> = RedBlackTree::new();

    //     for num in &nums {
    //         tree.insert(*num);
    //     }
    //     assert_eq!(tree.delete(&55), Some(55));
    //     println!("{:#?}", tree);
    // }
}
