extern mod btree;
extern mod extra;

use btree::{BTree};
use std::rand::{Rng, IsaacRng, SeedableRng};

use extra::test::BenchHarness;

use std::hashmap::HashMap;
use std::trie::TrieMap;
use extra::treemap::TreeMap;

fn check(btree: &BTree<uint, uint>, key: uint, expected: uint)
{
    match btree.find(&key) {
        Some(val) => {
            if *val != expected {
                fail!("{:?} != {:?} {:?}", *val, expected, btree);
            }
        },
        None => {
            fail!("key: {:?} not found in {:?}", key, btree);
        }
    }
}

fn shuffled(count: uint) -> ~[uint]
{
    let mut rng = IsaacRng::new();
    rng.reseed([60388u32]);

    let mut build_arr = ~[];

    for i in range(0, count) {
        build_arr.push(i as uint);
    }

    rng.shuffle_mut(build_arr);

    build_arr
}

fn insert_and_fetch_n(len: uint)
{
    let mut btree: BTree<uint, uint> = BTree::new();

    for i in range(0, len) {
        btree.insert(i, i);
    }


    for i in range(0, len) {
        check(&btree, i, i);
    }
    assert!(len as uint == btree.len());
}

fn insert_and_fetch_shuffle_n(len: uint)
{
    let mut btree: BTree<uint, uint> = BTree::new();

    let build_arr = shuffled(len as uint);


    for &b in build_arr.iter() {
        btree.insert(b, b);

    }

    for &b in build_arr.iter() {
        btree.insert(b, b);
    }

    for i in range(0, len) {
        check(&btree, i, i);
    }
    assert!(len as uint == btree.len());
}

fn update_shuffle_n(len: uint)
{
    let mut btree: BTree<uint, uint> = BTree::new();

    let build_arr = shuffled(len as uint);

    for &b in build_arr.iter() {
        assert!(btree.insert(b, b) == false);
    }

    for i in range(0, len) {
        check(&btree, i, i);
    }

    for &b in build_arr.iter() {
        assert!(btree.insert(b, (b+100)) == true);
    }

    for i in range(0, len) {
        check(&btree, i, i+100);
    }
    assert!(len as uint == btree.len());
}

fn remove_n(len: uint)
{
    let mut btree: BTree<uint, uint> = BTree::new();

    let build_arr = shuffled(len as uint);

    for &i in build_arr.iter() {
        btree.insert(i, i);
    }

    for &i in build_arr.iter() {
        check(&btree, i, i);
    }

    for &i in build_arr.iter() {
        if !btree.find(&i).is_some() {
            println!("{:?}, {:?}", i, btree);
        }
        assert!(btree.remove(&i) == true);
        assert!(btree.find(&i).is_none());
        assert!(btree.remove(&i) == false);
    }
    assert!(0 == btree.len())
}

fn pop_n(len: uint)
{
    let mut btree: BTree<uint, uint> = BTree::new();

    let build_arr = shuffled(len as uint);

    for &i in build_arr.iter() {
        btree.insert(i, i);
    }

    for &i in build_arr.iter() {
        check(&btree, i, i);
    }

    for &i in build_arr.iter() {
        assert!(btree.pop(&i).unwrap() == i);
        assert!(btree.find(&i).is_none());
    }

    assert!(0 == btree.len())
}

fn find_mut_n(len: uint)
{
    let mut btree: BTree<uint, uint> = BTree::new();

    let build_arr = shuffled(len as uint);

    for &i in build_arr.iter() {
        btree.insert(i, i);
    }

    for &i in build_arr.iter() {
        check(&btree, i, i);
    }

    for &i in build_arr.iter() {
        // stupid borrow checker
        {
            let out = btree.find_mut(&i).unwrap();
            assert!(*out == i);
            *out += 100;
        }

        let out = btree.find(&i).unwrap();
        assert!(*out == i + 100)
    }
    assert!(len as uint == btree.len())
}

fn swap_n(len: uint)
{
    let mut btree: BTree<uint, uint> = BTree::new();

    let build_arr = shuffled(len as uint);

    for &i in build_arr.iter() {
        btree.insert(i, i);
    }

    for &i in build_arr.iter() {
        check(&btree, i, i);
    }

    for &i in build_arr.iter() {
        let out = btree.swap(i, (i + 100)).unwrap();
        assert!(out == i);
        assert!(btree.swap(i, 0).unwrap() == i + 100)
    }
    assert!(len as uint == btree.len())
}

fn insert_remove_shuffle_n(len: uint)
{
    let mut btree: BTree<uint, uint> = BTree::new();

    let build_arr = shuffled((len*4) as uint);

    for i in range(0u, 4u) {
        for &b in build_arr.slice(len*i, len*(i+1)).iter() {
            btree.insert(b, b);
        }

        for &b in build_arr.slice(len*i, len*(i+1)).iter() {
            check(&btree, b, b);
        }
    }

    for &b in build_arr.slice(0, len*4).iter() {
        check(&btree, b, b);
    }

    for i in range(0u, 4u) {
        for &b in build_arr.slice(len*i, len*(i+1)).iter() {
            btree.remove(&b);
        }

        for &b in build_arr.slice(len*(i+1), len*4).iter() {
            check(&btree, b, b);
        }
    }
}

fn iter_test_n(len: uint)
{
    let mut btree: BTree<uint, uint> = BTree::new();
    let build_arr = shuffled(len as uint);

    let mut value_sum = 0;

    for &b in build_arr.iter() {
        btree.insert(b, b);
        value_sum += b;
    }

    for (&_, &v) in btree.iter() {
        value_sum -= v;
    }

    assert!(value_sum == 0);
}

#[test]
fn insert_and_fetch_10() { insert_and_fetch_n(10) }
#[test]
fn insert_and_fetch_80() { insert_and_fetch_n(80) }
#[test]
fn insert_and_fetch_120() { insert_and_fetch_n(120) }
#[test]
fn insert_and_fetch_990() { insert_and_fetch_n(990) }
#[test]
fn insert_and_fetch_2_500() { insert_and_fetch_n(2_500) }
#[test]
fn insert_and_fetch_10_000() { insert_and_fetch_n(10_000) }
#[test]
fn insert_and_fetch_40_000() { insert_and_fetch_n(40_000) }

#[test]
fn insert_and_fetch_shuffle_10() { insert_and_fetch_shuffle_n(10) }
#[test]
fn insert_and_fetch_shuffle_80() { insert_and_fetch_shuffle_n(80) }
#[test]
fn insert_and_fetch_shuffle_120() { insert_and_fetch_shuffle_n(120) }
#[test]
fn insert_and_fetch_shuffle_990() { insert_and_fetch_shuffle_n(990) }
#[test]
fn insert_and_fetch_shuffle_2_500() { insert_and_fetch_shuffle_n(2_500) }
#[test]
fn insert_and_fetch_shuffle_10_000() { insert_and_fetch_shuffle_n(10_000) }
#[test]
fn insert_and_fetch_shuffle_40_000() { insert_and_fetch_shuffle_n(40_000) }

#[test]
fn update_and_fetch_10() { update_shuffle_n(10) }
#[test]
fn update_and_fetch_80() { update_shuffle_n(80) }
#[test]
fn update_and_fetch_120() { update_shuffle_n(120) }
#[test]
fn update_and_fetch_990() { update_shuffle_n(990) }
#[test]
fn update_and_fetch_2_500() { update_shuffle_n(2_500) }
#[test]
fn update_and_fetch_10_000() { update_shuffle_n(10_000) }
#[test]
fn update_and_fetch_40_000() { update_shuffle_n(40_000) }

#[test]
fn remove_10() { remove_n(10) }
#[test]
fn remove_80() { remove_n(80) }
#[test]
fn remove_120() { remove_n(120) }
#[test]
fn remove_990() { remove_n(990) }
#[test]
fn remove_2_500() { remove_n(2_500) }
#[test]
fn remove_10_000() { remove_n(10_000) }
#[test]
fn remove_40_000() { remove_n(40_000) }

#[test]
fn pop_10() { pop_n(10) }
#[test]
fn pop_80() { pop_n(80) }
#[test]
fn pop_120() { pop_n(120) }
#[test]
fn pop_990() { pop_n(990) }
#[test]
fn pop_2_500() { pop_n(2_500) }
#[test]
fn pop_10_000() { pop_n(10_000) }
#[test]
fn pop_40_000() { pop_n(40_000) }

#[test]
fn find_mut_10() { find_mut_n(10) }
#[test]
fn find_mut_80() { find_mut_n(80) }
#[test]
fn find_mut_120() { find_mut_n(120) }
#[test]
fn find_mut_990() { find_mut_n(990) }
#[test]
fn find_mut_2_500() { find_mut_n(2_500) }
#[test]
fn find_mut_10_000() { find_mut_n(10_000) }
#[test]
fn find_mut_40_000() { find_mut_n(40_000) }

#[test]
fn swap_10() { swap_n(10) }
#[test]
fn swap_80() { swap_n(80) }
#[test]
fn swap_120() { swap_n(120) }
#[test]
fn swap_990() { swap_n(990) }
#[test]
fn swap_2_500() { swap_n(2_500) }
#[test]
fn swap_10_000() { swap_n(10_000) }
#[test]
fn swap_40_000() { swap_n(40_000) }

#[test]
fn insert_remove_shuffle_10() { insert_remove_shuffle_n(10) }
#[test]
fn insert_remove_shuffle_80() { insert_remove_shuffle_n(80) }
#[test]
fn insert_remove_shuffle_120() { insert_remove_shuffle_n(120) }
#[test]
fn insert_remove_shuffle_990() { insert_remove_shuffle_n(990) }
#[test]
fn insert_remove_shuffle_2_500() { insert_remove_shuffle_n(2_500) }
#[test]
fn insert_remove_shuffle_10_000() { insert_remove_shuffle_n(10_000) }
#[test]
fn insert_remove_shuffle_40_000() { insert_remove_shuffle_n(40_000) }

#[test]
fn iter_test_10() { iter_test_n(10) }
#[test]
fn iter_test_80() { iter_test_n(80) }
#[test]
fn iter_test_120() { iter_test_n(120) }
#[test]
fn iter_test_990() { iter_test_n(990) }
#[test]
fn iter_test_2_500() { iter_test_n(2_500) }
#[test]
fn iter_test_10_000() { iter_test_n(10_000) }
#[test]
fn iter_test_40_000() { iter_test_n(40_000) }

#[test]
fn freeze()
{
    let mut btree: BTree<uint, uint> = BTree::new();

    for i in range(0, 1100u) {
        btree.insert(i, i);
    }

    for i in range(0, 1100u) {
        check(&btree, i, i);
    }

    btree.freeze();

    for i in range(0, 1100u) {
        check(&btree, i, i);
    }    
}

#[test]
fn freeze_set()
{
    let mut btree: BTree<uint, uint> = BTree::new();

    for i in range(0, 1100u) {
        btree.insert(i, i);
    }

    for i in range(0, 1100u) {
        check(&btree, i, i);
    }

    btree.freeze();

    for i in range(1100u, 2200u) {
        btree.insert(i, i);
    }

    for i in range(0, 2200u) {
        check(&btree, i, i);
    } 
}

#[test]
fn freeze_set2()
{
    let mut btree: BTree<uint, uint> = BTree::new();

    for i in range(0, 1100u) {
        btree.insert(i, i);
    }

    for i in range(0, 1100u) {
        check(&btree, i, i);
    }

    btree.freeze();

    let old = btree.clone();

    for i in range(1100u, 2200u) {
        btree.insert(i, i);
        for j in range(0, i+1) {
            check(&btree, j, j);
        }
    }

    for i in range(0, 2200u) {
        check(&btree, i, i);
    }

    for i in range(0, 1100u) {
        check(&old, i, i);
    }

    for i in range(1100u, 2200u) {
        assert!(old.find(&i).is_none());
    } 
}


fn bench_insert_forward_n<T: MutableMap<uint, uint>>(bench: &mut BenchHarness, len: uint, new: || -> T)
{
    bench.iter(|| {
        let mut map = new();
        for b in range(0, len) {
            map.insert(b, b);
        }
    });
}

fn bench_insert_shuffle_n<T: MutableMap<uint, uint>>(bench: &mut BenchHarness, len: uint, new: || -> T)
{
    let build_arr = shuffled(len as uint);

    bench.iter(|| {
        let mut map = new();
        for &b in build_arr.iter() {
            map.insert(b, b);
        }
    });    
}

fn bench_update_forward_n<T: MutableMap<uint, uint>>(bench: &mut BenchHarness, len: uint, new: || -> T)
{
    let mut map = new();
    for b in range(0, len) {
        map.insert(b, b);
    }

    bench.iter(|| {
        for b in range(0, len) {
            map.insert(b, b+100);
        }
    });
}

fn bench_update_shuffle_n<T: MutableMap<uint, uint>>(bench: &mut BenchHarness, len: uint, new: || -> T)
{
    let build_arr = shuffled(len as uint);
    let mut map = new();
    for &b in build_arr.iter() {
        map.insert(b, b);
    }

    bench.iter(|| {
        for &b in build_arr.iter() {
            map.insert(b, b+100);
        }
    });
}

fn bench_get_forward_n<T: MutableMap<uint, uint>>(bench: &mut BenchHarness, len: uint, new: || -> T)
{
    let mut map = new();
    for b in range(0, len) {
        map.insert(b, b);
    }

    bench.iter(|| {
        for b in range(0, len) {
            map.find(&b);
        }
    });
}

fn bench_get_shuffle_n<T: MutableMap<uint, uint>>(bench: &mut BenchHarness, len: uint, new: || -> T)
{
    let build_arr = shuffled(len as uint);
    let mut map = new();
    for &b in build_arr.iter() {
        map.insert(b, b);
    }

    bench.iter(|| {
        for b in range(0, len) {
            map.find(&b);
        }
    });
}

fn bench_clone_n<T: MutableMap<uint, uint>+Clone>(bench: &mut BenchHarness, len: uint, new: || -> T)
{
    let build_arr = shuffled(len as uint);
    let mut map = new();
    for &b in build_arr.iter() {
        map.insert(b, b);
    }

    bench.iter(|| {
        let _ = map.clone();
    });
}


fn bench_remove_n<T: MutableMap<uint, uint>+Clone>(bench: &mut BenchHarness, len: uint, new: || -> T)
{
    let build_arr = shuffled(len as uint);
    let mut map = new();
    for &b in build_arr.iter() {
        map.insert(b, b);
    }

    bench.iter(|| {
        let mut map = map.clone();
        for &b in build_arr.iter() {
            map.remove(&b);
        }
    });
}

fn bench_swap_forward_n<T: MutableMap<uint, uint>>(bench: &mut BenchHarness, len: uint, new: || -> T)
{
    let mut map = new();
    for b in range(0, len) {
        map.insert(b, b);
    }

    bench.iter(|| {
        for b in range(0, len) {
            map.swap(b, b+100);
        }
    });
}

fn bench_swap_shuffle_n<T: MutableMap<uint, uint>>(bench: &mut BenchHarness, len: uint, new: || -> T)
{
    let build_arr = shuffled(len as uint);
    let mut map = new();
    for &b in build_arr.iter() {
        map.insert(b, b);
    }

    bench.iter(|| {
        for &b in build_arr.iter() {
            map.swap(b, b+100);
        }
    });
}

fn btree_bench_iter_n(bench: &mut BenchHarness, len: uint)
{
    let build_arr = shuffled(len as uint);
    let mut map: BTree<uint, uint> = BTree::new();
    for &b in build_arr.iter() {
        map.insert(b, b);
    }

    let mut k_sum = 0;
    let mut v_sum = 0;
    bench.iter(|| {
        for (k, v) in map.iter() {
            k_sum += *k;
            v_sum += *v;
        }
    });
}

fn hmap_bench_iter_n(bench: &mut BenchHarness, len: uint)
{
    let build_arr = shuffled(len as uint);
    let mut map: HashMap<uint, uint> = HashMap::new();
    for &b in build_arr.iter() {
        map.insert(b, b);
    }

    let mut k_sum = 0;
    let mut v_sum = 0;
    bench.iter(|| {
        for (k, v) in map.iter() {
            k_sum += *k;
            v_sum += *v;
        }
    });
}

fn tris_bench_iter_n(bench: &mut BenchHarness, len: uint)
{
    let build_arr = shuffled(len as uint);
    let mut map: TrieMap<uint> = TrieMap::new();
    for &b in build_arr.iter() {
        map.insert(b, b);
    }

    let mut k_sum = 0;
    let mut v_sum = 0;
    bench.iter(|| {
        for (k, v) in map.iter() {
            k_sum += k;
            v_sum += *v;
        }
    });
}

fn tmap_bench_iter_n(bench: &mut BenchHarness, len: uint)
{
    let build_arr = shuffled(len as uint);
    let mut map: TreeMap<uint, uint> = TreeMap::new();
    for &b in build_arr.iter() {
        map.insert(b, b);
    }

    let mut k_sum = 0;
    let mut v_sum = 0;
    bench.iter(|| {
        for (k, v) in map.iter() {
            k_sum += *k;
            v_sum += *v;
        }
    });
}

#[bench]
fn btree_insert_forward_10(bench: &mut BenchHarness) {bench_insert_forward_n(bench, 10, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_insert_forward_100(bench: &mut BenchHarness) {bench_insert_forward_n(bench, 100, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_insert_forward_1_000(bench: &mut BenchHarness) {bench_insert_forward_n(bench, 1_000, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_insert_forward_10_000(bench: &mut BenchHarness) {bench_insert_forward_n(bench, 10_000, || {let out: BTree<uint, uint> = BTree::new(); out})}

#[bench]
fn btree_insert_shuffle_10(bench: &mut BenchHarness) {bench_insert_shuffle_n(bench, 10, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_insert_shuffle_100(bench: &mut BenchHarness) {bench_insert_shuffle_n(bench, 100, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_insert_shuffle_1_000(bench: &mut BenchHarness) {bench_insert_shuffle_n(bench, 1_000, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_insert_shuffle_10_000(bench: &mut BenchHarness) {bench_insert_shuffle_n(bench, 10_000, || {let out: BTree<uint, uint> = BTree::new(); out})}

#[bench]
fn btree_update_forward_10(bench: &mut BenchHarness) {bench_update_forward_n(bench, 10, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_update_forward_100(bench: &mut BenchHarness) {bench_update_forward_n(bench, 100, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_update_forward_1_000(bench: &mut BenchHarness) {bench_update_forward_n(bench, 1_000, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_update_forward_10_000(bench: &mut BenchHarness) {bench_update_forward_n(bench, 10_000, || {let out: BTree<uint, uint> = BTree::new(); out})}

#[bench]
fn btree_update_shuffle_10(bench: &mut BenchHarness) {bench_update_shuffle_n(bench, 10, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_update_shuffle_100(bench: &mut BenchHarness) {bench_update_shuffle_n(bench, 100, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_update_shuffle_1_000(bench: &mut BenchHarness) {bench_update_shuffle_n(bench, 1_000, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_update_shuffle_10_000(bench: &mut BenchHarness) {bench_update_shuffle_n(bench, 10_000, || {let out: BTree<uint, uint> = BTree::new(); out})}

#[bench]
fn btree_get_forward_10(bench: &mut BenchHarness) {bench_get_forward_n(bench, 10, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_get_forward_100(bench: &mut BenchHarness) {bench_get_forward_n(bench, 100, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_get_forward_1_000(bench: &mut BenchHarness) {bench_get_forward_n(bench, 1_000, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_get_forward_10_000(bench: &mut BenchHarness) {bench_get_forward_n(bench, 10_000, || {let out: BTree<uint, uint> = BTree::new(); out})}

#[bench]
fn btree_get_shuffle_10(bench: &mut BenchHarness) {bench_get_shuffle_n(bench, 10, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_get_shuffle_100(bench: &mut BenchHarness) {bench_get_shuffle_n(bench, 100, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_get_shuffle_1_000(bench: &mut BenchHarness) {bench_get_shuffle_n(bench, 1_000, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_get_shuffle_10_000(bench: &mut BenchHarness) {bench_get_shuffle_n(bench, 10_000, || {let out: BTree<uint, uint> = BTree::new(); out})}

#[bench]
fn btree_clone_shuffle_10(bench: &mut BenchHarness) {bench_clone_n(bench, 10, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_clone_shuffle_100(bench: &mut BenchHarness) {bench_clone_n(bench, 100, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_clone_shuffle_1_000(bench: &mut BenchHarness) {bench_clone_n(bench, 1_000, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_clone_shuffle_10_000(bench: &mut BenchHarness) {bench_clone_n(bench, 10_000, || {let out: BTree<uint, uint> = BTree::new(); out})}

#[bench]
fn btree_remove_shuffle_10(bench: &mut BenchHarness) {bench_remove_n(bench, 10, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_remove_shuffle_100(bench: &mut BenchHarness) {bench_remove_n(bench, 100, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_remove_shuffle_1_000(bench: &mut BenchHarness) {bench_remove_n(bench, 1_000, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_remove_shuffle_10_000(bench: &mut BenchHarness) {bench_remove_n(bench, 10_000, || {let out: BTree<uint, uint> = BTree::new(); out})}

#[bench]
fn btree_swap_forward_10(bench: &mut BenchHarness) {bench_swap_forward_n(bench, 10, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_swap_forward_100(bench: &mut BenchHarness) {bench_swap_forward_n(bench, 100, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_swap_forward_1_000(bench: &mut BenchHarness) {bench_swap_forward_n(bench, 1_000, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_swap_forward_10_000(bench: &mut BenchHarness) {bench_swap_forward_n(bench, 10_000, || {let out: BTree<uint, uint> = BTree::new(); out})}

#[bench]
fn btree_swap_shuffle_10(bench: &mut BenchHarness) {bench_swap_shuffle_n(bench, 10, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_swap_shuffle_100(bench: &mut BenchHarness) {bench_swap_shuffle_n(bench, 100, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_swap_shuffle_1_000(bench: &mut BenchHarness) {bench_swap_shuffle_n(bench, 1_000, || {let out: BTree<uint, uint> = BTree::new(); out})}
#[bench]
fn btree_swap_shuffle_10_000(bench: &mut BenchHarness) {bench_swap_shuffle_n(bench, 10_000, || {let out: BTree<uint, uint> = BTree::new(); out})}

#[bench]
fn btree_iter_10(bench: &mut BenchHarness) {btree_bench_iter_n(bench, 10);}
#[bench]
fn btree_iter_100(bench: &mut BenchHarness) {btree_bench_iter_n(bench, 100);}
#[bench]
fn btree_iter_1_000(bench: &mut BenchHarness) {btree_bench_iter_n(bench, 1_000);}
#[bench]
fn btree_iter_10_000(bench: &mut BenchHarness) {btree_bench_iter_n(bench, 10_000);}
#[bench]
fn btree_iter_100_000(bench: &mut BenchHarness) {btree_bench_iter_n(bench, 100_000);}

#[bench]
fn hmap_insert_forward_10(bench: &mut BenchHarness) {bench_insert_forward_n(bench, 10, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_insert_forward_100(bench: &mut BenchHarness) {bench_insert_forward_n(bench, 100, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_insert_forward_1_000(bench: &mut BenchHarness) {bench_insert_forward_n(bench, 1_000, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_insert_forward_10_000(bench: &mut BenchHarness) {bench_insert_forward_n(bench, 10_000, || {let out: HashMap<uint, uint> = HashMap::new(); out})}

#[bench]
fn hmap_insert_shuffle_10(bench: &mut BenchHarness) {bench_insert_shuffle_n(bench, 10, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_insert_shuffle_100(bench: &mut BenchHarness) {bench_insert_shuffle_n(bench, 100, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_insert_shuffle_1_000(bench: &mut BenchHarness) {bench_insert_shuffle_n(bench, 1_000, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_insert_shuffle_10_000(bench: &mut BenchHarness) {bench_insert_shuffle_n(bench, 10_000, || {let out: HashMap<uint, uint> = HashMap::new(); out})}

#[bench]
fn hmap_update_forward_10(bench: &mut BenchHarness) {bench_update_forward_n(bench, 10, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_update_forward_100(bench: &mut BenchHarness) {bench_update_forward_n(bench, 100, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_update_forward_1_000(bench: &mut BenchHarness) {bench_update_forward_n(bench, 1_000, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_update_forward_10_000(bench: &mut BenchHarness) {bench_update_forward_n(bench, 10_000, || {let out: HashMap<uint, uint> = HashMap::new(); out})}

#[bench]
fn hmap_update_shuffle_10(bench: &mut BenchHarness) {bench_update_shuffle_n(bench, 10, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_update_shuffle_100(bench: &mut BenchHarness) {bench_update_shuffle_n(bench, 100, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_update_shuffle_1_000(bench: &mut BenchHarness) {bench_update_shuffle_n(bench, 1_000, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_update_shuffle_10_000(bench: &mut BenchHarness) {bench_update_shuffle_n(bench, 10_000, || {let out: HashMap<uint, uint> = HashMap::new(); out})}

#[bench]
fn hmap_get_forward_10(bench: &mut BenchHarness) {bench_get_forward_n(bench, 10, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_get_forward_100(bench: &mut BenchHarness) {bench_get_forward_n(bench, 100, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_get_forward_1_000(bench: &mut BenchHarness) {bench_get_forward_n(bench, 1_000, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_get_forward_10_000(bench: &mut BenchHarness) {bench_get_forward_n(bench, 10_000, || {let out: HashMap<uint, uint> = HashMap::new(); out})}

#[bench]
fn hmap_get_shuffle_10(bench: &mut BenchHarness) {bench_get_shuffle_n(bench, 10, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_get_shuffle_100(bench: &mut BenchHarness) {bench_get_shuffle_n(bench, 100, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_get_shuffle_1_000(bench: &mut BenchHarness) {bench_get_shuffle_n(bench, 1_000, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_get_shuffle_10_000(bench: &mut BenchHarness) {bench_get_shuffle_n(bench, 10_000, || {let out: HashMap<uint, uint> = HashMap::new(); out})}

#[bench]
fn hmap_clone_shuffle_10(bench: &mut BenchHarness) {bench_clone_n(bench, 10, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_clone_shuffle_100(bench: &mut BenchHarness) {bench_clone_n(bench, 100, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_clone_shuffle_1_000(bench: &mut BenchHarness) {bench_clone_n(bench, 1_000, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_clone_shuffle_10_000(bench: &mut BenchHarness) {bench_clone_n(bench, 10_000, || {let out: HashMap<uint, uint> = HashMap::new(); out})}

#[bench]
fn hmap_remove_shuffle_10(bench: &mut BenchHarness) {bench_remove_n(bench, 10, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_remove_shuffle_100(bench: &mut BenchHarness) {bench_remove_n(bench, 100, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_remove_shuffle_1_000(bench: &mut BenchHarness) {bench_remove_n(bench, 1_000, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_remove_shuffle_10_000(bench: &mut BenchHarness) {bench_remove_n(bench, 10_000, || {let out: HashMap<uint, uint> = HashMap::new(); out})}

#[bench]
fn hmap_swap_forward_10(bench: &mut BenchHarness) {bench_swap_forward_n(bench, 10, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_swap_forward_100(bench: &mut BenchHarness) {bench_swap_forward_n(bench, 100, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_swap_forward_1_000(bench: &mut BenchHarness) {bench_swap_forward_n(bench, 1_000, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_swap_forward_10_000(bench: &mut BenchHarness) {bench_swap_forward_n(bench, 10_000, || {let out: HashMap<uint, uint> = HashMap::new(); out})}

#[bench]
fn hmap_swap_shuffle_10(bench: &mut BenchHarness) {bench_swap_shuffle_n(bench, 10, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_swap_shuffle_100(bench: &mut BenchHarness) {bench_swap_shuffle_n(bench, 100, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_swap_shuffle_1_000(bench: &mut BenchHarness) {bench_swap_shuffle_n(bench, 1_000, || {let out: HashMap<uint, uint> = HashMap::new(); out})}
#[bench]
fn hmap_swap_shuffle_10_000(bench: &mut BenchHarness) {bench_swap_shuffle_n(bench, 10_000, || {let out: HashMap<uint, uint> = HashMap::new(); out})}

#[bench]
fn hmap_iter_10(bench: &mut BenchHarness) {hmap_bench_iter_n(bench, 10);}
#[bench]
fn hmap_iter_100(bench: &mut BenchHarness) {hmap_bench_iter_n(bench, 100);}
#[bench]
fn hmap_iter_1_000(bench: &mut BenchHarness) {hmap_bench_iter_n(bench, 1_000);}
#[bench]
fn hmap_iter_10_000(bench: &mut BenchHarness) {hmap_bench_iter_n(bench, 10_000);}
#[bench]
fn hmap_iter_100_000(bench: &mut BenchHarness) {hmap_bench_iter_n(bench, 100_000);}

#[bench]
fn tris_insert_forward_10(bench: &mut BenchHarness) {bench_insert_forward_n(bench, 10, || {let out: TrieMap<uint> = TrieMap::new(); out})}
#[bench]
fn tris_insert_forward_100(bench: &mut BenchHarness) {bench_insert_forward_n(bench, 100, || {let out: TrieMap<uint> = TrieMap::new(); out})}
#[bench]
fn tris_insert_forward_1_000(bench: &mut BenchHarness) {bench_insert_forward_n(bench, 1_000, || {let out: TrieMap<uint> = TrieMap::new(); out})}
#[bench]
fn tris_insert_forward_10_000(bench: &mut BenchHarness) {bench_insert_forward_n(bench, 10_000, || {let out: TrieMap<uint> = TrieMap::new(); out})}

#[bench]
fn tris_insert_shuffle_10(bench: &mut BenchHarness) {bench_insert_shuffle_n(bench, 10, || {let out: TrieMap<uint> = TrieMap::new(); out})}
#[bench]
fn tris_insert_shuffle_100(bench: &mut BenchHarness) {bench_insert_shuffle_n(bench, 100, || {let out: TrieMap<uint> = TrieMap::new(); out})}
#[bench]
fn tris_insert_shuffle_1_000(bench: &mut BenchHarness) {bench_insert_shuffle_n(bench, 1_000, || {let out: TrieMap<uint> = TrieMap::new(); out})}
#[bench]
fn tris_insert_shuffle_10_000(bench: &mut BenchHarness) {bench_insert_shuffle_n(bench, 10_000, || {let out: TrieMap<uint> = TrieMap::new(); out})}

#[bench]
fn tris_update_forward_10(bench: &mut BenchHarness) {bench_update_forward_n(bench, 10, || {let out: TrieMap<uint> = TrieMap::new(); out})}
#[bench]
fn tris_update_forward_100(bench: &mut BenchHarness) {bench_update_forward_n(bench, 100, || {let out: TrieMap<uint> = TrieMap::new(); out})}
#[bench]
fn tris_update_forward_1_000(bench: &mut BenchHarness) {bench_update_forward_n(bench, 1_000, || {let out: TrieMap<uint> = TrieMap::new(); out})}
#[bench]
fn tris_update_forward_10_000(bench: &mut BenchHarness) {bench_update_forward_n(bench, 10_000, || {let out: TrieMap<uint> = TrieMap::new(); out})}

#[bench]
fn tris_update_shuffle_10(bench: &mut BenchHarness) {bench_update_shuffle_n(bench, 10, || {let out: TrieMap<uint> = TrieMap::new(); out})}
#[bench]
fn tris_update_shuffle_100(bench: &mut BenchHarness) {bench_update_shuffle_n(bench, 100, || {let out: TrieMap<uint> = TrieMap::new(); out})}
#[bench]
fn tris_update_shuffle_1_000(bench: &mut BenchHarness) {bench_update_shuffle_n(bench, 1_000, || {let out: TrieMap<uint> = TrieMap::new(); out})}
#[bench]
fn tris_update_shuffle_10_000(bench: &mut BenchHarness) {bench_update_shuffle_n(bench, 10_000, || {let out: TrieMap<uint> = TrieMap::new(); out})}

#[bench]
fn tris_get_forward_10(bench: &mut BenchHarness) {bench_get_forward_n(bench, 10, || {let out: TrieMap<uint> = TrieMap::new(); out})}
#[bench]
fn tris_get_forward_100(bench: &mut BenchHarness) {bench_get_forward_n(bench, 100, || {let out: TrieMap<uint> = TrieMap::new(); out})}
#[bench]
fn tris_get_forward_1_000(bench: &mut BenchHarness) {bench_get_forward_n(bench, 1_000, || {let out: TrieMap<uint> = TrieMap::new(); out})}
#[bench]
fn tris_get_forward_10_000(bench: &mut BenchHarness) {bench_get_forward_n(bench, 10_000, || {let out: TrieMap<uint> = TrieMap::new(); out})}

#[bench]
fn tris_get_shuffle_10(bench: &mut BenchHarness) {bench_get_shuffle_n(bench, 10, || {let out: TrieMap<uint> = TrieMap::new(); out})}
#[bench]
fn tris_get_shuffle_100(bench: &mut BenchHarness) {bench_get_shuffle_n(bench, 100, || {let out: TrieMap<uint> = TrieMap::new(); out})}
#[bench]
fn tris_get_shuffle_1_000(bench: &mut BenchHarness) {bench_get_shuffle_n(bench, 1_000, || {let out: TrieMap<uint> = TrieMap::new(); out})}
#[bench]
fn tris_get_shuffle_10_000(bench: &mut BenchHarness) {bench_get_shuffle_n(bench, 10_000, || {let out: TrieMap<uint> = TrieMap::new(); out})}

#[bench]
fn tris_iter_10(bench: &mut BenchHarness) {tris_bench_iter_n(bench, 10);}
#[bench]
fn tris_iter_100(bench: &mut BenchHarness) {tris_bench_iter_n(bench, 100);}
#[bench]
fn tris_iter_1_000(bench: &mut BenchHarness) {tris_bench_iter_n(bench, 1_000);}
#[bench]
fn tris_iter_10_000(bench: &mut BenchHarness) {tris_bench_iter_n(bench, 10_000);}
#[bench]
fn tris_iter_100_000(bench: &mut BenchHarness) {tris_bench_iter_n(bench, 100_000);}

#[bench]
fn tmap_iter_10(bench: &mut BenchHarness) {tmap_bench_iter_n(bench, 10);}
#[bench]
fn tmap_iter_100(bench: &mut BenchHarness) {tmap_bench_iter_n(bench, 100);}
#[bench]
fn tmap_iter_1_000(bench: &mut BenchHarness) {tmap_bench_iter_n(bench, 1_000);}
#[bench]
fn tmap_iter_10_000(bench: &mut BenchHarness) {tmap_bench_iter_n(bench, 10_000);}
#[bench]
fn tmap_iter_100_000(bench: &mut BenchHarness) {tmap_bench_iter_n(bench, 100_000);}
