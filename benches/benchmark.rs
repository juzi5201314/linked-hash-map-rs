use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    insert_group(c);
    get_group(c);
    clear_group(c);
    pop_group(c);
    remove_group(c);
}

fn remove_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove");
    let mut linked_hash_map_rs = gen_1k_linked_hash_map_rs();
    let mut linked_hash_map = gen_1k_linked_hash_map();

    group.bench_function(BenchmarkId::new("linked_hash_map_rs", "1k"), |b| {
        b.iter(|| {
            for i in 0..1_000 {
                black_box(linked_hash_map_rs.remove(black_box(&i)));
            }
        })
    });

    group.bench_function(BenchmarkId::new("linked_hash_map", "1k"), |b| {
        b.iter(|| {
            for i in 0..1_000 {
                black_box(linked_hash_map.remove(black_box(&i)));
            }
        })
    });

    group.finish();
}

fn pop_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("pop");

    group.bench_function(BenchmarkId::new("linked_hash_map_rs", "1k"), |b| {
        b.iter_batched(
            gen_1k_linked_hash_map_rs,
            |mut map| black_box(map.pop_back()),
            BatchSize::SmallInput,
        )
    });

    group.bench_function(BenchmarkId::new("linked_hash_map", "1k"), |b| {
        b.iter_batched(
            gen_1k_linked_hash_map,
            |mut map| black_box(map.pop_back()),
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn clear_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("clear");

    group.bench_function(BenchmarkId::new("linked_hash_map_rs", ""), |b| {
        b.iter_batched(
            gen_1k_linked_hash_map_rs,
            |mut map| map.clear(),
            BatchSize::SmallInput,
        )
    });

    group.bench_function(BenchmarkId::new("linked_hash_map", ""), |b| {
        b.iter_batched(
            gen_1k_linked_hash_map,
            |mut map| map.clear(),
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn get_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("get");
    let linked_hash_map_rs = gen_1k_linked_hash_map_rs();
    let linked_hash_map = gen_1k_linked_hash_map();

    group.bench_function(BenchmarkId::new("linked_hash_map_rs", "1k"), |b| {
        b.iter(|| {
            for i in 0..1_000 {
                black_box(linked_hash_map_rs.get(black_box(&i)));
            }
        })
    });

    group.bench_function(BenchmarkId::new("linked_hash_map", "1k"), |b| {
        b.iter(|| {
            for i in 0..1_000 {
                black_box(linked_hash_map.get(black_box(&i)));
            }
        })
    });

    group.finish();
}

fn insert_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");

    group.bench_function(BenchmarkId::new("linked_hash_map_rs", "1k"), |b| {
        b.iter_batched(
            || linked_hash_map_rs::LinkedHashMap::with_capacity(1_000),
            |mut map| {
                for i in 0..1_000 {
                    map.insert(black_box(i), black_box(i));
                }
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function(BenchmarkId::new("linked_hash_map", "1k"), |b| {
        b.iter_batched(
            || linked_hash_map::LinkedHashMap::with_capacity(1_000),
            |mut map| {
                for i in 0..1_000 {
                    map.insert(black_box(i), black_box(i));
                }
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn gen_1k_linked_hash_map_rs() -> linked_hash_map_rs::LinkedHashMap<usize, usize> {
    let mut linked_hash_map_rs = linked_hash_map_rs::LinkedHashMap::with_capacity(1_000);
    for i in 0..1_000 {
        linked_hash_map_rs.insert(black_box(i), black_box(i));
    }
    linked_hash_map_rs
}

fn gen_1k_linked_hash_map() -> linked_hash_map::LinkedHashMap<usize, usize> {
    let mut linked_hash_map = linked_hash_map::LinkedHashMap::with_capacity(1_000);
    for i in 0..1_000 {
        linked_hash_map.insert(black_box(i), black_box(i));
    }
    linked_hash_map
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(1000);
    targets = criterion_benchmark);
criterion_main!(benches);
