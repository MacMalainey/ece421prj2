pub mod avl;
pub mod tree;
pub mod redblack;
mod unbalanced; // Left private because if anyone really requires it, they can just use the typedef

// Typedefs for easy access
pub type Tree<T, U> = tree::Tree<T, U>;
pub type AVLTree<T> = Tree<T, avl::AVLBalance>;
pub type RedBlackTree<T> = Tree<T, redblack::RedBlackBalance>;
pub type BinarySearchTree<T> = Tree<T, unbalanced::UnbalancedBalance>;

#[cfg(test)]
mod tests {
    use super::{AVLTree, RedBlackTree};

    #[test]
    #[ignore]
    // Always passes, used for manual verification and inspection
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
    #[ignore]
    // Always passes, used for manual verification and inspection
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
    #[ignore]
    // Always passes, used for manual verification and inspection
    fn manual_rb_check_insert() {
        let nums = [
            40, 50, 60, 70, 90, 80, 100, 110
            // 40, 65, 55,
            // 57, 58, 75, 60, 59
        ];
        let mut tree: RedBlackTree<u32> = RedBlackTree::new();

        for num in &nums {
            println!("----- Insert: {} -------", num);
            tree.insert(*num);
            println!("{:#?}", tree);
        }
    }

    #[test]
    // #[ignore]
    // Always passes, used for manual verification and inspection
    fn manual_rb_check_delete() {
        let nums = [
            40, 50, 60, 70, 90, 80
        ];
        let mut tree: RedBlackTree<u32> = RedBlackTree::new();

        for num in &nums {
            tree.insert(*num);
        }
        assert_eq!(tree.delete(&90), Some(90));
        assert_eq!(tree.delete(&80), Some(80));
        assert_eq!(tree.delete(&40), Some(40));
        println!("{:#?}", tree);
    }
}
