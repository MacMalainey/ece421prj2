use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use project2::{RedBlackTree, AVLTree};
use rand::Rng;

fn generate_values(limit: usize) -> Vec<u32> {
    let mut values = Vec::with_capacity(limit);
    let mut rng = rand::thread_rng();
    let mut next = 0;
    for i in 0..limit {
        values.insert(i, next);
        next += rng.gen_range(1..=4);
    }
    values
}

fn bench_balanced_tree(c: &mut Criterion) {
    let mut group = c.benchmark_group("Balanced Tree Insert + Search");
    let data = generate_values(130000);
    for tree_size in &[10000, 40000, 70000, 100000, 130000] {
        // Benchmark Red Black Tree
        group.bench_with_input(
            BenchmarkId::new("Red Black Tree", tree_size),
            tree_size,
            |b, num| {
                b.iter_with_large_drop(|| {
                    let mut tree = RedBlackTree::new();
                    // Insert tree_size elements into tree
                    data.iter().take(*num).for_each(|v| tree.insert(*v));
                    // Search for the first tree_size/10 elements in the tree
                    data.iter().take(num/10).for_each(|v| { tree.search(&v); });
                    tree
                })
            }
        );

        // Benchmark AVL Tree
        group.bench_with_input(
            BenchmarkId::new("AVL Tree", tree_size),
            tree_size,
            |b, num| {
                b.iter_with_large_drop(|| {
                    let mut tree = AVLTree::new();
                    // Insert tree_size elements into tree
                    data.iter().take(*num).for_each(|v| tree.insert(*v));
                    // Search for the first tree_size/10 elements in the tree
                    data.iter().take(num/10).for_each(|v| { tree.search(&v); });
                    tree
                })
            }
        );
    }
    group.finish()
}

criterion_group!(benches, bench_balanced_tree);
criterion_main!(benches);