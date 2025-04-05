use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use avl_tree::avl::AVLTree;
use rand::{thread_rng, Rng};
use std::iter;

fn generate_random_string(length: usize) -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(rand::distributions::Alphanumeric))
        .map(char::from)
        .take(length)
        .collect()
}

fn avl_set_benchmark(c: &mut Criterion) {
    let mut tree = AVLTree::new();
    let mut group = c.benchmark_group("AVL Tree Operations");
    group.throughput(Throughput::Elements(100_000));
    
    group.bench_function("set ops/sec", |b| {
        b.iter(|| {
            for _ in 0..100_000 {
                let key = generate_random_string(6);
                let value = generate_random_string(6);
                tree.set(&key, &value);
            }
        })
    });
    group.finish();
}

fn avl_get_benchmark(c: &mut Criterion) {
    let mut tree = AVLTree::new();
    
    // Pre-fill the tree with random strings
    for _ in 0..100_000 {
        let key = generate_random_string(6);
        let value = generate_random_string(6);
        tree.set(&key, &value);
    }
    
    let mut group = c.benchmark_group("AVL Tree Operations");
    group.throughput(Throughput::Elements(100_000));
    
    group.bench_function("get ops/sec", |b| {
        b.iter(|| {
            for _ in 0..100_000 {
                let key = generate_random_string(6);
                tree.get(&key);
            }
        })
    });
    group.finish();
}

fn avl_unset_benchmark(c: &mut Criterion) {
    let mut tree = AVLTree::new();
    
    // Pre-fill the tree with random strings
    for _ in 0..100_000 {
        let key = generate_random_string(6);
        let value = generate_random_string(6);

        tree.set(&key, &value);
    }
    
    let mut group = c.benchmark_group("AVL Tree Operations");
    group.throughput(Throughput::Elements(100_000));
    
    group.bench_function("unset ops/sec", |b| {
        b.iter(|| {
            for _ in 0..100_000 {
                let key = generate_random_string(6);
                tree.unset(&key);
            }
        })
    });
    group.finish();
}

criterion_group!(benches, avl_set_benchmark, avl_get_benchmark, avl_unset_benchmark);
criterion_main!(benches); 