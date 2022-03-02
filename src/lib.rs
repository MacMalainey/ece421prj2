mod avl;
mod tree;
mod redblack;

pub type Tree<T, U> = tree::Tree<T, U>;
pub type AVLTree<T> = Tree<T, avl::AVLBranch<T>>;

#[cfg(test)]
mod tests {
    use super::AVLTree;

    #[test]
    fn manual_check_order() {
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
    fn manual_check_delete() {
        let nums = [
            40, 65, 55,
            57, 58, 75, 60, 59
        ];
        let mut tree: AVLTree<u32> = AVLTree::new();

        for num in &nums {
            tree.insert(*num);
        }
        println!("----- Insert: {} -------", 40);
        assert_eq!(tree.delete(&40), Some(40));
        println!("{:#?}", tree);
    }
}
