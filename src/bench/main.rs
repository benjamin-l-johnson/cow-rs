#[feature(macro_rules)];

extern mod btree;
extern mod extra;

use btree::BTree;
use extra::time::precise_time_s;
use std::rand::{Rng, IsaacRng, SeedableRng};
use std::hashmap::HashMap;
use std::trie::TrieMap;
use extra::treemap::TreeMap;

#[inline(always)]
fn time(f: ||) -> f64
{
    let start = precise_time_s();
    f();
    let end = precise_time_s();
    (end - start)
}

#[inline(always)]
fn timed_repeat<T>(cycles: uint, count: uint, setup: |uint| -> T, timed: |uint, &T|) -> f64
{
    let out = setup(count);
    time(|| {
        for _ in range(0, cycles) {
            timed(count, &out);
        }
    }) / (cycles as f64)
}

#[inline(always)]
fn timed<T>(count: uint, setup: |uint| -> T, timed: |uint, &T|) -> f64
{
    let out = setup(count);
    let t = time(||{
        timed(count, &out);
    });

    if t < 1. {
        timed_repeat((1./t) as uint, count, setup, timed)
    } else {
        t
    }
}

#[inline(always)]
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

#[inline(always)]
fn forward(count: uint) -> ~[uint]
{
    let mut build_arr = ~[];

    for i in range(0, count) {
        build_arr.push(i as uint);
    }

    build_arr
}

#[inline(always)]
fn btree_build(build_arr: ~[uint]) -> (~[uint], BTree<uint, uint>)
{
    let mut btree = BTree::new();
    for &node in build_arr.iter() {
        btree.insert(node, node);
    }
    (build_arr, btree)
}


#[inline(always)]
fn btree_insert(_: uint, data: &~[uint])
{
    let mut btree = BTree::new();
    for &node in data.iter() {
        btree.insert(node, node);
    }
}

#[inline(always)]
fn btree_find(_: uint, tup: &(~[uint], BTree<uint, uint>))
{
    match *tup {
        (ref data, ref btree) => {
            for node in data.iter() {
                assert!(btree.find(node).is_some());
            }     
        }
    }
}

#[inline(always)]
fn btree_clone(_: uint, tup: &(~[uint], BTree<uint, uint>))
{
    match *tup {
        (_, ref btree) => {
           let _ = btree.clone();   
        }
    }
}

#[inline(always)]
fn btree_iter(_: uint, tup: &(~[uint], BTree<uint, uint>))
{
    match *tup {
        (_, ref btree) => {
            let mut count = 1;
            for (&key, &value) in btree.iter() {
                count += key;
                count += value;
            }
            assert!(count != 0);
        }
    }
}

#[inline(always)]
fn hmap_build(build_arr: ~[uint]) -> (~[uint], HashMap<uint, uint>)
{
    let mut hmap = HashMap::new();
    for &node in build_arr.iter() {
        hmap.insert(node, node);
    }

    (build_arr, hmap)
}


#[inline(always)]
fn hmap_insert(_: uint, data: &~[uint])
{
    let mut hmap = HashMap::new();
    for &node in data.iter() {
        hmap.insert(node, node);
    }
}

#[inline(always)]
fn hmap_find(_: uint, tup: &(~[uint], HashMap<uint, uint>))
{
    match *tup {
        (ref data, ref hmap) => {
            for node in data.iter() {
                assert!(hmap.find(node).is_some());
            }     
        }
    }
}

#[inline(always)]
fn hmap_clone(_: uint, tup: &(~[uint], HashMap<uint, uint>))
{
    match *tup {
        (_, ref hmap) => {
           let _ = hmap.clone();   
        }
    }
}

#[inline(always)]
fn hmap_iter(_: uint, tup: &(~[uint], HashMap<uint, uint>))
{
    match *tup {
        (_, ref hmap) => {
            let mut count = 1;
            for (&key, &value) in hmap.iter() {
                count += key;
                count += value
            }
            assert!(count != 0);
        }
    }
}


#[inline(always)]
fn tmap_build(build_arr: ~[uint]) -> (~[uint], TreeMap<uint, uint>)
{
    let mut tmap = TreeMap::new();
    for &node in build_arr.iter() {
        tmap.insert(node, node);
    }

    (build_arr, tmap)
}


#[inline(always)]
fn tmap_insert(_: uint, data: &~[uint])
{
    let mut tmap = TreeMap::new();
    for &node in data.iter() {
        tmap.insert(node, node);
    }
}

#[inline(always)]
fn tmap_find(_: uint, tup: &(~[uint], TreeMap<uint, uint>))
{
    match *tup {
        (ref data, ref tmap) => {
            for node in data.iter() {
                assert!(tmap.find(node).is_some());
            }     
        }
    }
}

#[inline(always)]
fn tmap_clone(_: uint, tup: &(~[uint], TreeMap<uint, uint>))
{
    match *tup {
        (_, ref hmap) => {
           let _ = hmap.clone();   
        }
    }
}

#[inline(always)]
fn tmap_iter(_: uint, tup: &(~[uint], TreeMap<uint, uint>))
{
    match *tup {
        (_, ref tmap) => {
            let mut count = 1;
            for (&key, &value) in tmap.iter() {
                count += key;
                count += value
            }
            assert!(count != 0);
        }
    }
}

#[inline(always)]
fn trie_build(build_arr: ~[uint]) -> (~[uint], TrieMap<uint>)
{
    let mut trie = TrieMap::new();
    for &node in build_arr.iter() {
        trie.insert(node, node);
    }

    (build_arr, trie)
}


#[inline(always)]
fn trie_insert(_: uint, data: &~[uint])
{
    let mut trie = TrieMap::new();
    for &node in data.iter() {
        trie.insert(node, node);
    }
}

#[inline(always)]
fn trie_find(_: uint, tup: &(~[uint], TrieMap<uint>))
{
    match *tup {
        (ref data, ref trie) => {
            for node in data.iter() {
                assert!(trie.find(node).is_some());
            }     
        }
    }
}

#[inline(always)]
fn trie_clone(_: uint, tup: &(~[uint], TrieMap<uint>))
{
    match *tup {
        (_, ref trie) => {
           let _ = trie.clone();   
        }
    }
}

#[inline(always)]
fn trie_iter(_: uint, tup: &(~[uint], TrieMap<uint>))
{
    match *tup {
        (_, ref trie) => {
            let mut count = 1;
            for (key, &value) in trie.iter() {
                count += key;
                count += value
            }
            assert!(count != 0);
        }
    }
}

struct BenchResult {
    size: uint,
    insert_shuffle: f64,
    insert_forward: f64,
    find_shuffled: f64,
    find_forward: f64,
    clone: f64,
    iter: f64
}

struct Bench {
    time: f64,
    name: ~str,
    result: ~[BenchResult]
}

fn benchwalk(size: uint, f: |uint|)
{
    for i in range(0, size) {
        f(1<<i);
        if i > 1 {
            f((1<<i) + (1<<(i-1)));
        }
    }
}

#[inline(always)]
fn bench<T>(name: ~str, build: |~[uint]| -> T, insert: |uint, &~[uint]|, find: |uint, &T|, clone: |uint, &T|, iter: |uint, &T|) -> Bench
{
    let every = 26;

    let mut out = ~[];

    let t = time(|| {
        benchwalk(every, |size| {
            let res = BenchResult {
                size: size,
                insert_forward: timed(size, forward, |x, y|{insert(x, y)}),
                insert_shuffle: timed(size, shuffled, |x, y|{insert(x, y)}),
                find_shuffled: timed(size, |size| {build(shuffled(size))}, |x, y|{find(x, y)}),
                find_forward: timed(size, |size| {build(forward(size))}, |x, y|{find(x, y)}),
                clone: timed(size, |size| {build(shuffled(size))}, |x, y|{clone(x, y)}),
                iter: timed(size, |size| {build(forward(size))}, |x, y|{iter(x, y)})
            };
            out.push(res);
        });
    });

    println!("{:s},{:4.1}", name.clone(), t);

    Bench {
        time: t,
        name: name,
        result: out
    }
}

macro_rules! print_table_resp_s(
    ($name:expr, $table:expr, $row:ident) => ({
        println!("{:s}", $name);
        for x in $table.slice(0, 1).iter() {
            print!("size,");
            for y in x.result.iter() {
                print!("{:0.0},", y.size);
            }
            println!("");
        }
        for x in $table.iter() {
            print!("{},", x.name);
            for y in x.result.iter() {
                print!("{:0.0},", y.size as f64 / y.$row);
            }
            println!("");
        }
        println("");
    })
)

macro_rules! print_table_abs(
    ($name:expr, $table:expr, $row:ident) => ({
        println!("{:s}", $name);
        for x in $table.slice(0, 1).iter() {
            print!("size,");
            for y in x.result.iter() {
                print!("{:0.0},", y.size);
            }
            println!("");
        }
        for x in $table.iter() {
            print!("{},", x.name);
            for y in x.result.iter() {
                print!("{:0.9},", y.$row);
            }
            println!("");
        }
        println("");
    })
)

fn main()
{
    let mut table = ~[];
    table.push(bench(~"Btree", btree_build, btree_insert, btree_find, btree_clone, btree_iter));
    table.push(bench(~"HashMap", hmap_build, hmap_insert, hmap_find, hmap_clone, hmap_iter));
    table.push(bench(~"TreeMap", tmap_build, tmap_insert, tmap_find, tmap_clone, tmap_iter));
    table.push(bench(~"TrieMap", trie_build, trie_insert, trie_find, trie_clone, trie_iter));

    print_table_resp_s!("insert shuffled", table, insert_shuffle);
    print_table_resp_s!("insert forward", table, insert_forward);
    print_table_resp_s!("find shuffled", table, find_shuffled)
    print_table_resp_s!("find forward", table, find_forward);
    print_table_resp_s!("clone", table, clone);
    print_table_resp_s!("iterator", table, iter);

    print_table_abs!("insert shuffled", table, insert_shuffle);
    print_table_abs!("insert forward", table, insert_forward);
    print_table_abs!("find shuffled", table, find_shuffled)
    print_table_abs!("find forward", table, find_forward);
    print_table_abs!("clone", table, clone);
    print_table_abs!("iterator", table, iter);
}