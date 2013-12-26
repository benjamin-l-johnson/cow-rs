extern mod btree;
extern mod extra;

use btree::BTree;
use extra::time::precise_time_s;
use std::rand::{Rng, IsaacRng, SeedableRng};

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
fn bench<T>(name: &str, build: |~[uint]| -> T, insert: |uint, &~[uint]|, find: |uint, &T|, clone: |uint, &T|, iter: |uint, &T|)
{
    let count = 26;

    println!("|{:s}|", name);
    print!("|size           |");
    for i in range(0, count) {
        let count = 1<<i;
        print!("{:9.0}|", count);
    }
    println!("");

    print!("|insert shuffled|");
    for i in range(0, count) {
        let count = 1<<i;
        print!("{:9.0}|", (count as f64) / timed(count, shuffled, |x, y|{insert(x, y)}));
    }
    println!("");

    print!("|find shuffled  |");
    for i in range(0, count) {
        let count = 1<<i;
        print!("{:9.0}|", (count as f64) / timed(count, |count| {build(shuffled(count))}, |x, y|{find(x, y)}));
    }
    println!("");

    print!("|insert forward |");
    for i in range(0, count) {
        let count = 1<<i;
        print!("{:9.0}|", (count as f64) / timed(count, forward, |x, y|{insert(x, y)}));
    }
    println!("");

    print!("|find forward   |");
    for i in range(0, count) {
        let count = 1<<i;
        print!("{:9.0}|", (count as f64) / timed(count, |count| {build(forward(count))}, |x, y|{find(x, y)}));
    }
    println!("");

    print!("|clone          |");
    for i in range(0, count) {
        let count = 1<<i;
        print!("{:9.0}|", (count as f64) / timed(count, |count| {build(shuffled(count))}, |x, y|{clone(x, y)}));
    }
    println!("");

    print!("|iter forward   |");
    for i in range(0, count) {
        let count = 1<<i;
        print!("{:9.0}|", (count as f64) / timed(count, |count| {build(forward(count))}, |x, y|{iter(x, y)}));
    }
    println!("");
}

fn main()
{
    bench("BTree", btree_build, btree_insert, btree_find, btree_clone, btree_iter);
    bench("HashMap", hmap_build, hmap_insert, hmap_find, hmap_clone, hmap_iter);
    bench("TreeMap", tmap_build, tmap_insert, tmap_find, tmap_clone, tmap_iter);
}