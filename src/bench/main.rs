#[feature(macro_rules)];

extern mod btree;
extern mod extra;

use btree::BTree;
use extra::time::precise_time_s;
use std::rand::{Rng, IsaacRng, SeedableRng};
use std::iter::range_step;
use std::hashmap::HashMap;
use extra::treemap::TreeMap;

#[inline(always)]
fn timed_repeat<T>(cycles: uint, count: uint, setup: |uint| -> T, timed: |uint, &T|) -> f64
{
    let out = setup(count);
    let start = precise_time_s();
    for _ in range(0, cycles) {
        timed(count, &out);
    }
    let end = precise_time_s();
    (end - start) / (cycles as f64)
}

#[inline(always)]
fn timed<T>(count: uint, setup: |uint| -> T, timed: |uint, &T|) -> f64
{
    let out = setup(count);
    let start = precise_time_s();
    timed(count, &out);
    let end = precise_time_s();

    if end-start < 1. {
        timed_repeat((1./(end-start)) as uint, count, setup, timed)
    } else {
        end - start
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
fn btree_freeze_build(build_arr: ~[uint]) -> (~[uint], BTree<uint, uint>)
{
    let mut btree = BTree::new();
    for &node in build_arr.iter() {
        btree.insert(node, node);
    }
    btree.freeze();
    (build_arr, btree)
}


#[inline(always)]
fn btree_freeze_insert(_: uint, data: &~[uint])
{
    let mut btree = BTree::new();
    for &node in data.iter() {
        btree.insert(node, node);
    }
    btree.freeze();
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
    name: ~str,
    result: ~[BenchResult]
}

#[inline(always)]
fn bench<T>(name: ~str, build: |~[uint]| -> T, insert: |uint, &~[uint]|, find: |uint, &T|, clone: |uint, &T|, iter: |uint, &T|) -> Bench
{
    let every = 16;
    let every_other = 26;

    let mut out = ~[];

    for size in range(0, every) {
        let size = 1<<size;
        let res = BenchResult{
            size: size,
            insert_forward: (size as f64) / timed(size, forward, |x, y|{insert(x, y)}),
            insert_shuffle: (size as f64) / timed(size, shuffled, |x, y|{insert(x, y)}),
            find_shuffled: (size as f64) / timed(size, |size| {build(shuffled(size))}, |x, y|{find(x, y)}),
            find_forward: (size as f64) / timed(size, |size| {build(forward(size))}, |x, y|{find(x, y)}),
            clone: (size as f64) / timed(size, |size| {build(shuffled(size))}, |x, y|{clone(x, y)}),
            iter: (size as f64) / timed(size, |size| {build(forward(size))}, |x, y|{iter(x, y)})
        };

        out.push(res);
    }

    for size in range_step(every, every_other, 2) {
        let size = 1<<size;
        let res = BenchResult{
            size: size,
            insert_forward: (size as f64) / timed(size, forward, |x, y|{insert(x, y)}),
            insert_shuffle: (size as f64) / timed(size, shuffled, |x, y|{insert(x, y)}),
            find_shuffled: (size as f64) / timed(size, |size| {build(shuffled(size))}, |x, y|{find(x, y)}),
            find_forward: (size as f64) / timed(size, |size| {build(forward(size))}, |x, y|{find(x, y)}),
            clone: (size as f64) / timed(size, |size| {build(shuffled(size))}, |x, y|{clone(x, y)}),
            iter: (size as f64) / timed(size, |size| {build(forward(size))}, |x, y|{iter(x, y)})
        };

        out.push(res);
    }

    Bench {
        name: name,
        result: out
    }
}

macro_rules! print_table(
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
                print!("{:0.0},", y.$row);
            }
            println!("");
        }
        println("");
    })
)

fn main()
{
    let mut  table = ~[];
    table.push(bench(~"Btree", btree_build, btree_insert, btree_find, btree_clone, btree_iter));
    table.push(bench(~"Btree frozen", btree_freeze_build, btree_freeze_insert, btree_find, btree_clone, btree_iter));
    table.push(bench(~"HashMap", hmap_build, hmap_insert, hmap_find, hmap_clone, hmap_iter));
    table.push(bench(~"TreeMap", tmap_build, tmap_insert, tmap_find, tmap_clone, tmap_iter));

    print_table!("insert shuffled", table, insert_shuffle);
    print_table!("insert forward", table, insert_forward);
    print_table!("find shuffled", table, find_shuffled)
    print_table!("find forward", table, find_forward);
    print_table!("clone", table, clone);
    print_table!("iterator", table, iter);
}