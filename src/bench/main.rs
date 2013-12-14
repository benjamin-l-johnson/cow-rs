extern mod btree;
extern mod extra;

use std::comm::{Port, Chan, stream, SharedChan};
use std::rand::{Rng, IsaacRng, SeedableRng};

use btree::BTree;
use extra::time::precise_time_ns;

use extra::arc::Arc;

fn time_to_nice(time: u64) -> ~str
{
    if time > 2_000_000_000 {
        format!("{:2.3f}s", time as f64 / 1_000_000_000.0)
    } else if time > 2_000_000 {
        format!("{:2.3f}ms", time as f64 / 1_000_000.0)
    } else if time > 2_000 {
        format!("{:2.3f}us", time as f64 / 1_000.0)
    } else {
        format!("{:}ns", time)
    }
}

fn main()
{
    let setup = precise_time_ns();
    let mut rng = IsaacRng::new();
    rng.reseed([60387u32]);

    let mut build_arr = ~[];

    for i in range(0, 25_000) {
        build_arr.push(i as int);
    }

    rng.shuffle_mut(build_arr);

    let build = precise_time_ns();
    let mut btree: BTree<int, int> = BTree::new();

    for &b in build_arr.iter() {
        btree.insert(b, b);
    }

    let build_arr = Arc::new(build_arr);

    let freeze = precise_time_ns();

    btree.freeze();

    let done_freeze = precise_time_ns();

    let (port, chan): (std::comm::Port<()>, std::comm::Chan<()>) = std::comm::stream();
    let chan = SharedChan::new(chan);

    let threads = 1_000_000_000 / 25_000;

    for _ in range(0, threads) {
        let btree = btree.clone();
        let chan = chan.clone();
        let build_arr = build_arr.clone();
        do std::task::spawn {
            for i in build_arr.get().iter() {
                btree.get(i);
            }
            chan.send(());
        }
    }

    for _ in range(0, threads) {
        port.recv();
    }

    let done_search = precise_time_ns();


    println!("setup: {}", time_to_nice(build - setup));
    println!("build: {}", time_to_nice(freeze - build));
    println!("freeze: {}", time_to_nice(done_freeze - freeze));
    println!("send: {} {:2.2}ns/get", time_to_nice(done_search - done_freeze), (done_search - done_freeze) as f64 / 1_000_000_000.);
    println!("stat: {:?}", btree.stat());
}