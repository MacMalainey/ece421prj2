extern crate project2;

use std::fmt::Debug;
use std::io;
use std::io::Write;

use project2::avl::AVLBalance;
use project2::redblack::RedBlackBalance;
use project2::tree::inspect::TreeBalance;
use project2::tree::Tree;

#[derive(strum_macros::Display)]
enum TreeType {
    RedBlack,
    AVL,
}

/// Get user input from stdin and split into a vector of strings
fn get_user_input() -> Vec<String> {
    print!("> ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok().expect("stdin error");

    let mut result: Vec<String> = Vec::new();

    for i in input.split_whitespace() {
        result.push(String::from(i));
    }

    return result;
}

/// Attempt to get which tree the user wants to use
fn get_tree_selection() -> TreeType {
    println!("Please pick a tree to test. Enter `1' to test a RedBlack tree or \
            `2' to test an AVL tree.");

    loop {
        let input = get_user_input();

        if input.len() <= 0 {
            continue;
        } else if input.len() >= 2 {
            println!("Too many arguments!");
            continue;
        }

        let command = &input[0];

        if command == "exit" {
            std::process::exit(0);
        } else if command == "1" {
            return TreeType::RedBlack;
        } else if command == "2" {
            return TreeType::AVL;
        } else {
            println!("Invalid command!");
        }
    }
}

/// Output help menu
fn print_help() {
    println!("List of commands:");
    println!(" - insert <n> ...... insert <n> into the tree");
    println!(" - delete <n> ...... delete <n> from the tree");
    println!(" - search <n> ...... output whether <n> is in the tree or not");
    println!(" - clear ........... clear all elements from the tree");
    println!(" - isempty ......... output whether the tree is empty or not");
    println!(" - height .......... output the height of the tree");
    println!(" - leaves .......... output the number of leaves in the tree");
    println!(" - print tree ...... print formatted tree");
    println!(" - print inorder ... print elements using inorder traversal");
    println!(" - switch .......... select a new tree type");
    println!(" - help ............ reprint this menu");
    println!(" - exit ............ exit program");
}

/// Once the tree is selected, we start taking user input to manipulate the tree
fn manipulate_tree<U: TreeBalance + Debug>(mut tree: Tree<u32, U>) {
    print_help();

    loop {
        let input = get_user_input();

        if input.len() <= 0 {
            continue;
        } else if input.len() >= 3 {
            println!("Too many arguments!");
            continue;
        }

        let command = &input[0];

        if command == "insert" {
            if input.len() >= 2 {
                match &input[1].parse::<u32>() {
                    Ok(n) => {
                        tree.insert(*n);
                        println!("Inserted {}.", n);
                    },
                    Err(_) => {
                        println!("Enter an unsigned integer.");
                    },
                };
            } else {
                println!("Missing argument!");
            }
        } else if command == "delete" {
            if input.len() >= 2 {
                match &input[1].parse::<u32>() {
                    Ok(n) => {
                        match tree.delete(n) {
                            None => {
                                println!("Did not find {}.", n);
                            }
                            Some(_) => {
                                println!("Deleted {}.", n);
                            }
                        }
                    },
                    Err(_) => {
                        println!("Enter an unsigned integer.")
                    },
                };
            } else {
                println!("Missing argument!");
            }
        } else if command == "search" {
            if input.len() >= 2 {
                match &input[1].parse::<u32>() {
                    Ok(n) => {
                        if tree.search(n) {
                            println!("Found {}!", n);
                        } else {
                            println!("Did not find {}.", n);
                        }
                    },
                    Err(_) => {
                        println!("Enter an unsigned integer.")
                    },
                };
            } else {
                println!("Missing argument!");
            }
        } else if command == "clear" {
            tree.clear();
            println!("Tree has been cleared.");
        } else if command == "isempty" {
            println!("Is empty: {}", tree.is_empty());
        } else if command == "height" {
            println!("Height: {}", tree.height());
        } else if command == "leaves" {
            println!("Leaves: {}", tree.leaves());
        } else if command == "print" {
            if input.len() >= 2 {
                let argument = &input[1];

                if argument == "tree" {
                    println!("{:#?}", tree);
                } else if argument == "inorder" {
                    println!("{}", tree);
                } else {
                    println!("Invalid argument!");
                }
            } else {
                println!("Missing argument!");
            }
        } else if command == "switch" {
            break;
        } else if command == "help" {
            print_help();
        } else if command == "exit" {
            std::process::exit(0);
        } else {
            println!("Invalid command!");
        }
    }
}

fn main() {
    println!("Welcome!");
    println!("This is the CLI for testing RedBlack and AVL trees.");
    println!("Enter `exit' to exit at any point.");

    loop {
        let tree_selection = get_tree_selection();
        println!("You have selected a {} tree.", tree_selection.to_string());

        match tree_selection {
            TreeType::RedBlack => manipulate_tree::<RedBlackBalance>(Tree::new()),
            TreeType::AVL => manipulate_tree::<AVLBalance>(Tree::new()),
        }
    }
}
