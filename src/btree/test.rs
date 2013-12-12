extern mod btree;
extern mod extra;

use btree::BTree;
use std::rand::{Rng, IsaacRng, SeedableRng};

use extra::test::BenchHarness;


fn check(btree: &BTree<int, int>, key: int, expected: int)
{
    match btree.get(&key) {
        Some(val) => {
            if *val != expected {
                fail!("{:?} != {:?}", val, expected);
            }
        },
        None => {
            fail!("key: {:?} not found in {:?}", key, btree);
        }
    }
}


#[test]
fn insert_and_fetch_10()
{
    let mut btree: BTree<int, int> = BTree::new();

    for i in range(0, 10) {
        btree.set(&i, &i);
    }


    for i in range(0, 10) {
        check(&btree, i, i);
    }
}

#[test]
fn insert_and_fetch_80()
{
    let mut btree: BTree<int, int> = BTree::new();

    for i in range(0, 80) {
        btree.set(&i, &i);
    }

    for i in range(0, 80) {
        check(&btree, i, i);
    }
}

#[test]
fn insert_and_fetch_120()
{
    let mut btree: BTree<int, int> = BTree::new();

    for i in range(0, 120) {
        btree.set(&i, &i);
    }

    for i in range(0, 120) {
        check(&btree, i, i);
    }
}

#[test]
fn insert_and_fetch_990()
{
    let mut btree: BTree<int, int> = BTree::new();

    for i in range(0, 990) {
        btree.set(&i, &i);
    }

    for i in range(0, 990) {
        check(&btree, i, i);
    }
}


#[test]
fn insert_and_fetch_1100()
{
    let mut btree: BTree<int, int> = BTree::new();

    for i in range(0, 1100) {
        btree.set(&i, &i);
        for j in range(0, i+1) {
            check(&btree, j, j);
        }
    }

    for i in range(0, 1100) {
        check(&btree, i, i);
    }
}

#[test]
fn insert_and_fetch_5K()
{
    let mut btree: BTree<int, int> = BTree::new();

    for i in range(0, 5_000) {
        btree.set(&i, &i);
    }

    for i in range(0, 5_000) {
        check(&btree, i, i);
    }
}

#[test]
fn insert_and_fetch_20K()
{
    let mut btree: BTree<int, int> = BTree::new();

    for i in range(0, 20_000) {
        btree.set(&i, &i);
    }

    for i in range(0, 20_000) {
        check(&btree, i, i);
    }
}

#[test]
fn insert_and_fetch_shuffle_10()
{
    let mut btree: BTree<int, int> = BTree::new();

    let mut rng = IsaacRng::new();
    rng.reseed([60387u32]);

    let mut build_arr = ~[];

    for i in range(0, 10) {
        build_arr.push(i);
    }

    rng.shuffle_mut(build_arr);

    for b in build_arr.iter() {
        btree.set(b, b);
    }

    for i in range(0, 10) {
        check(&btree, i, i);
    }
}


#[test]
fn insert_and_fetch_shuffle_80()
{
    let mut btree: BTree<int, int> = BTree::new();

    let mut rng = IsaacRng::new();
    rng.reseed([60387u32]);

    let mut build_arr = ~[];

    for i in range(0, 80) {
        build_arr.push(i);
    }

    rng.shuffle_mut(build_arr);

    for b in build_arr.iter() {
        btree.set(b, b);

    }

    for i in range(0, 80) {
        check(&btree, i, i);
    }
}

#[test]
fn insert_and_fetch_shuffle_120()
{
    let mut btree: BTree<int, int> = BTree::new();

    let mut rng = IsaacRng::new();
    rng.reseed([60387u32]);

    let mut build_arr = ~[];

    for i in range(0, 120) {
        build_arr.push(i);
    }

    rng.shuffle_mut(build_arr);

    for b in build_arr.iter() {
        btree.set(b, b);

    }

    for i in range(0, 120) {
        check(&btree, i, i);
    }
}


#[test]
fn insert_and_fetch_shuffle_990()
{
    let mut btree: BTree<int, int> = BTree::new();

    let mut rng = IsaacRng::new();
    rng.reseed([60387u32]);

    let mut build_arr = ~[];

    for i in range(0, 990) {
        build_arr.push(i);
    }

    rng.shuffle_mut(build_arr);

    for b in build_arr.iter() {
        btree.set(b, b);
    }

    for i in range(0, 990) {
        check(&btree, i, i);
    }
}


#[test]
fn insert_and_fetch_shuffle_1100()
{
    let mut btree: BTree<int, int> = BTree::new();

    let mut rng = IsaacRng::new();
    rng.reseed([60387u32]);

    let mut build_arr = ~[];

    for i in range(0, 1100) {
        build_arr.push(i);
    }

    rng.shuffle_mut(build_arr);

    for b in build_arr.iter() {
        btree.set(b, b);
    }

    for i in range(0, 1100) {
        check(&btree, i, i);
    }
}

#[test]
fn insert_and_fetch_shuffle_5K()
{
    let mut btree: BTree<int, int> = BTree::new();

    let mut rng = IsaacRng::new();
    rng.reseed([60387u32]);

    let mut build_arr = ~[];

    for i in range(0, 5_000) {
        build_arr.push(i);
    }

    rng.shuffle_mut(build_arr);

    for b in build_arr.iter() {
        btree.set(b, b);
    }

    for i in range(0, 5_000) {
        check(&btree, i, i);
    }
}

#[test]
fn insert_and_fetch_shuffle_20K()
{
    let mut btree: BTree<int, int> = BTree::new();

    let mut rng = IsaacRng::new();
    rng.reseed([60387u32]);

    let mut build_arr = ~[];

    for i in range(0, 20_000) {
        build_arr.push(i);
    }

    rng.shuffle_mut(build_arr);

    for b in build_arr.iter() {
        btree.set(b, b);
    }

    for i in range(0, 20_000) {
        check(&btree, i, i);
    }
}

#[bench]
fn btree_bench_insert_1K(bench: &mut BenchHarness)
{
    let mut rng = IsaacRng::new();
    rng.reseed([60387u32]);

    let mut build_arr = ~[];

    for i in range(0, 1_000) {
        build_arr.push(i);
    }

    rng.shuffle_mut(build_arr);

    bench.iter(|| {
        let mut btree: BTree<int, int> = BTree::new();
        for b in build_arr.iter() {
            btree.set(b, b);
        }
    });
}


#[bench]
fn btree_bench_get_1K(bench: &mut BenchHarness)
{
    let mut rng = IsaacRng::new();
    rng.reseed([60387u32]);

    let mut build_arr = ~[];

    for i in range(0, 1_000) {
        build_arr.push(i);
    }

    rng.shuffle_mut(build_arr);

    let mut btree: BTree<int, int> = BTree::new();
    for b in build_arr.iter() {
        btree.set(b, b);
    }

    bench.iter(|| {
        for b in build_arr.iter() {
            btree.get(b);
        }
    });
}


#[bench]
fn btree_bench_get_5K(bench: &mut BenchHarness)
{
    let mut rng = IsaacRng::new();
    rng.reseed([60387u32]);

    let mut build_arr = ~[];

    for i in range(0, 5_000) {
        build_arr.push(i);
    }

    rng.shuffle_mut(build_arr);

    let mut btree: BTree<int, int> = BTree::new();
    for b in build_arr.iter() {
        btree.set(b, b);
    }

    bench.iter(|| {
        for b in build_arr.iter() {
            btree.get(b);
        }
    });
}

#[bench]
fn btree_bench_linear_5K(bench: &mut BenchHarness)
{
    let mut btree: BTree<int, int> = BTree::new();

    for i in range(0, 10_000) {
        btree.set(&i, &i);
    }

    bench.iter(|| {
        for i in range(0, 5_000) {
            btree.get(&i);
        }
    });
}

#[bench]
fn hmap_bench_linear_5K(bench: &mut BenchHarness)
{
    let mut hmap: std::hashmap::HashMap<int, int> = std::hashmap::HashMap::new();

    for i in range(0, 10_000) {
        hmap.insert(i, i);
    }

    bench.iter(|| {
        for i in range(0, 5_000) {
            hmap.get(&i);
        }
    });
}


#[bench]
fn table_bench_linear_5K(bench: &mut BenchHarness)
{
    let mut table: ~[(int, Option<int>)] = ~[];

    for i in range(0, 10_000) {
        table.push((i, Some(i)));
    }

    bench.iter(|| {
        for i in range(0, 5_000) {
            table.bsearch(|&(k, _)| {i.cmp(&k)});
        }
    });
}

#[bench]
fn hmap_bench_get_1K(bench: &mut BenchHarness)
{
    let mut rng = IsaacRng::new();
    rng.reseed([60387u32]);

    let mut build_arr = ~[];

    for i in range(0, 1_000) {
        build_arr.push(i);
    }

    rng.shuffle_mut(build_arr);

    let mut hmap: std::hashmap::HashMap<int, int> = std::hashmap::HashMap::new();
    for b in build_arr.iter() {
        hmap.insert(*b, *b);
    }

    bench.iter(|| {
        for b in build_arr.iter() {
            hmap.get(b);
        }
    });
}

#[bench]
fn hmap_bench_get_5K(bench: &mut BenchHarness)
{
    let mut rng = IsaacRng::new();
    rng.reseed([60387u32]);

    let mut build_arr = ~[];

    for i in range(0, 5_000) {
        build_arr.push(i);
    }

    rng.shuffle_mut(build_arr);

    let mut hmap: std::hashmap::HashMap<int, int> = std::hashmap::HashMap::new();
    for b in build_arr.iter() {
        hmap.insert(*b, *b);
    }

    bench.iter(|| {
        for b in build_arr.iter() {
            hmap.get(b);
        }
    });
}


#[bench]
fn hmap_bench_insert_1K(bench: &mut BenchHarness)
{
    let mut rng = IsaacRng::new();
    rng.reseed([60387u32]);

    let mut build_arr = ~[];

    for i in range(0, 1_000) {
        build_arr.push(i);
    }

    rng.shuffle_mut(build_arr);

    bench.iter(|| {
        let mut btree: std::hashmap::HashMap<int, int> = std::hashmap::HashMap::new();
        for b in build_arr.iter() {
            btree.insert(*b, *b);
        }
    });
}
