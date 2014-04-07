extern crate cow;
extern crate rand;

mod btree {
    use cow::btree::{BTreeMap};
    use rand::{Rng, IsaacRng, SeedableRng};
    static NUM_TASKS: uint = 8;

    fn check(btree: &BTreeMap<uint, uint>, key: uint, expected: uint)
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
        let mut rng = IsaacRng::new().unwrap();
        rng.reseed([60388u32]);

        let mut build_arr = ~[];

        for i in range(0, count) {
            build_arr.push(i as uint);
        }

        rng.shuffle(build_arr);

        build_arr
    }

    fn insert_and_fetch_n(len: uint)
    {
        let mut btree: BTreeMap<uint, uint> = BTreeMap::new();

        for i in range(0, len) {
            btree.insert(i, i);
            assert!(i+1 == btree.len())
        }

        for i in range(0, len) {
            check(&btree, i, i);
        }
        assert!(len as uint == btree.len());
    }

    fn insert_and_fetch_shuffle_n(len: uint)
    {
        let mut btree: BTreeMap<uint, uint> = BTreeMap::new();

        let build_arr = shuffled(len as uint);

        for (idx, &b) in build_arr.iter().enumerate() {
            btree.insert(b, b);
            assert!(idx+1 == btree.len())
        }

        for i in range(0, len) {
            check(&btree, i, i);
        }
        assert!(len as uint == btree.len());
    }

    fn update_shuffle_n(len: uint)
    {
        let mut btree: BTreeMap<uint, uint> = BTreeMap::new();

        let build_arr = shuffled(len as uint);

        for (idx, &b) in build_arr.iter().enumerate() {
            assert!(btree.insert(b, b) == false);
            assert!(idx+1 == btree.len());
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
        let mut btree: BTreeMap<uint, uint> = BTreeMap::new();

        let build_arr = shuffled(len as uint);

        for (cnt, &i) in build_arr.iter().enumerate() {
            btree.insert(i, i);
            assert!(cnt+1 == btree.len());
        }

        for &i in build_arr.iter() {
            check(&btree, i, i);
        }

        for (cnt, &i) in build_arr.iter().enumerate() {
            assert!(btree.remove(&i) == true);
            assert!(btree.find(&i).is_none());
            assert!(btree.remove(&i) == false);
            assert!(len-cnt-1 == btree.len());
        }
        assert!(0 == btree.len())
    }

    fn pop_n(len: uint)
    {
        let mut btree: BTreeMap<uint, uint> = BTreeMap::new();

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
        let mut btree: BTreeMap<uint, uint> = BTreeMap::new();

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
        let mut btree: BTreeMap<uint, uint> = BTreeMap::new();

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
        let mut btree: BTreeMap<uint, uint> = BTreeMap::new();

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
        let mut btree: BTreeMap<uint, uint> = BTreeMap::new();
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
    fn insert_and_fetch_100_000() { insert_and_fetch_n(100_000) }

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
    fn insert_and_fetch_shuffle_100_000() { insert_and_fetch_shuffle_n(100_000) }

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
    fn update_and_fetch_100_000() { update_shuffle_n(100_000) }

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
    fn remove_100_000() { remove_n(100_000) }

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
    fn pop_100_000() { pop_n(100_000) }

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
    fn find_mut_100_000() { find_mut_n(100_000) }

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
    fn swap_100_000() { swap_n(100_000) }

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
    fn insert_remove_shuffle_100_000() { insert_remove_shuffle_n(100_000) }

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
    fn iter_test_100_000() { iter_test_n(100_000) }

    #[test]
    fn cow_clone()
    {
        let mut btree: BTreeMap<uint, uint> = BTreeMap::new();

        for i in range(0, 1100u) {
            btree.insert(i, i);
        }

        for i in range(0, 1100u) {
            check(&btree, i, i);
        }

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

    fn cow_tasks_append_n(count: uint)
    {
        let mut btree: BTreeMap<uint, uint> = BTreeMap::new();

        for i in range(0, count) {
            btree.insert(i, i);
        }

        for i in range(0, count) {
            check(&btree, i, i);
        }

        for offset in range(1u, NUM_TASKS+1) {
            let new = btree.clone();
            spawn(proc() {
                let mut new = new;
                for i in range(count, count*2) {
                    new.insert(i, i+offset);
                }
                for i in range(0, count) {
                    check(&new, i, i);
                }
                for i in range(count, count*2) {
                    check(&new, i, i+offset);
                }
            });
        }

        for i in range(0, count) {
            check(&btree, i, i);
        }

        for i in range(count, count*2) {
            assert!(btree.find(&i).is_none());
        } 
    }

    fn cow_tasks_update_n(count: uint)
    {
        let mut btree: BTreeMap<uint, uint> = BTreeMap::new();

        for i in range(0, count) {
            btree.insert(i, i);
        }

        for i in range(0, count) {
            check(&btree, i, i);
        }

        for offset in range(1u, NUM_TASKS+1) {
            let new = btree.clone();
            spawn(proc() {
                let mut new = new;
                for i in range(0, count) {
                    new.insert(i, i+offset);
                }
                for i in range(0, count) {
                    check(&new, i, i+offset);
                }
            });
        }

        for i in range(0, count) {
            check(&btree, i, i);
        } 
    }

    fn cow_tasks_remove_n(count: uint)
    {
        let mut btree: BTreeMap<uint, uint> = BTreeMap::new();

        for i in range(0, count) {
            btree.insert(i, i);
        }

        for i in range(0, count) {
            check(&btree, i, i);
        }

        for _ in range(1u, NUM_TASKS+1) {
            let new = btree.clone();
            spawn(proc() {
                let mut new = new;
                for i in range(0, count) {
                    new.remove(&i);
                }

                for i in range(0, count) {
                    assert!(new.find(&i).is_none());
                } 
            });
        }

        for i in range(0, count) {
            check(&btree, i, i);
        } 
    }

    fn cow_tasks_swap_n(count: uint)
    {
        let mut btree: BTreeMap<uint, uint> = BTreeMap::new();

        for i in range(0, count) {
            btree.insert(i, i);
        }

        for i in range(0, count) {
            check(&btree, i, i);
        }

        for offset in range(1u, NUM_TASKS+1) {
            let new = btree.clone();
            spawn(proc() {
                let mut new = new;
                for i in range(0, count) {
                    assert!(new.swap(i, i+offset).is_some());
                }
                for i in range(0, count) {
                    check(&new, i, i+offset);
                }
            });
        }

        for i in range(0, count) {
            check(&btree, i, i);
        } 
    }

    fn cow_tasks_pop_n(count: uint)
    {
        let mut btree: BTreeMap<uint, uint> = BTreeMap::new();

        for i in range(0, count) {
            btree.insert(i, i);
        }

        for i in range(0, count) {
            check(&btree, i, i);
        }

        for _ in range(1u, NUM_TASKS+1) {
            let new = btree.clone();
            spawn(proc() {
                let mut new = new;
                for i in range(0, count) {
                    assert!(new.pop(&i).is_some());
                }
                for i in range(0, count) {
                    assert!(new.pop(&i).is_none());
                }
            });
        }

        for i in range(0, count) {
            check(&btree, i, i);
        } 
    }

    fn cow_tasks_find_mut_n(count: uint)
    {
        let mut btree: BTreeMap<uint, uint> = BTreeMap::new();

        for i in range(0, count) {
            btree.insert(i, i);
        }

        for i in range(0, count) {
            check(&btree, i, i);
        }

        for offset in range(1u, NUM_TASKS+1) {
            let new = btree.clone();
            spawn(proc() {
                let mut new = new;
                for i in range(0, count) {
                    let val = new.find_mut(&i).unwrap();
                    *val = i + offset;
                }
                for i in range(0, count) {
                    check(&new, i, i+offset);
                }
            });
        }

        for i in range(0, count) {
            check(&btree, i, i);
        } 
    }

    #[test]
    fn cow_tasks_append_10() { cow_tasks_append_n(10) }
    #[test]
    fn cow_tasks_append_80() { cow_tasks_append_n(80) }
    #[test]
    fn cow_tasks_append_120() { cow_tasks_append_n(120) }
    #[test]
    fn cow_tasks_append_990() { cow_tasks_append_n(990) }
    #[test]
    fn cow_tasks_append_2_500() { cow_tasks_append_n(2_500) }
    #[test]
    fn cow_tasks_append_10_000() { cow_tasks_append_n(10_000) }
    #[test]
    fn cow_tasks_append_100_000() { cow_tasks_append_n(100_000) }

    #[test]
    fn cow_tasks_update_10() { cow_tasks_update_n(10) }
    #[test]
    fn cow_tasks_update_80() { cow_tasks_update_n(80) }
    #[test]
    fn cow_tasks_update_120() { cow_tasks_update_n(120) }
    #[test]
    fn cow_tasks_update_990() { cow_tasks_update_n(990) }
    #[test]
    fn cow_tasks_update_2_500() { cow_tasks_update_n(2_500) }
    #[test]
    fn cow_tasks_update_10_000() { cow_tasks_update_n(10_000) }
    #[test]
    fn cow_tasks_update_100_000() { cow_tasks_update_n(100_000) }

    #[test]
    fn cow_tasks_remove_10() { cow_tasks_remove_n(10) }
    #[test]
    fn cow_tasks_remove_80() { cow_tasks_remove_n(80) }
    #[test]
    fn cow_tasks_remove_120() { cow_tasks_remove_n(120) }
    #[test]
    fn cow_tasks_remove_990() { cow_tasks_remove_n(990) }
    #[test]
    fn cow_tasks_remove_2_500() { cow_tasks_remove_n(2_500) }
    #[test]
    fn cow_tasks_remove_10_000() { cow_tasks_remove_n(10_000) }
    #[test]
    fn cow_tasks_remove_100_000() { cow_tasks_remove_n(100_000) }

    #[test]
    fn cow_tasks_swap_10() { cow_tasks_swap_n(10) }
    #[test]
    fn cow_tasks_swap_80() { cow_tasks_swap_n(80) }
    #[test]
    fn cow_tasks_swap_120() { cow_tasks_swap_n(120) }
    #[test]
    fn cow_tasks_swap_990() { cow_tasks_swap_n(990) }
    #[test]
    fn cow_tasks_swap_2_500() { cow_tasks_swap_n(2_500) }
    #[test]
    fn cow_tasks_swap_10_000() { cow_tasks_swap_n(10_000) }
    #[test]
    fn cow_tasks_swap_100_000() { cow_tasks_swap_n(100_000) }

    #[test]
    fn cow_tasks_pop_10() { cow_tasks_pop_n(10) }
    #[test]
    fn cow_tasks_pop_80() { cow_tasks_pop_n(80) }
    #[test]
    fn cow_tasks_pop_120() { cow_tasks_pop_n(120) }
    #[test]
    fn cow_tasks_pop_990() { cow_tasks_pop_n(990) }
    #[test]
    fn cow_tasks_pop_2_500() { cow_tasks_pop_n(2_500) }
    #[test]
    fn cow_tasks_pop_10_000() { cow_tasks_pop_n(10_000) }
    #[test]
    fn cow_tasks_pop_100_000() { cow_tasks_pop_n(100_000) }

    #[test]
    fn cow_tasks_find_mut_10() { cow_tasks_find_mut_n(10) }
    #[test]
    fn cow_tasks_find_mut_80() { cow_tasks_find_mut_n(80) }
    #[test]
    fn cow_tasks_find_mut_120() { cow_tasks_find_mut_n(120) }
    #[test]
    fn cow_tasks_find_mut_990() { cow_tasks_find_mut_n(990) }
    #[test]
    fn cow_tasks_find_mut_2_500() { cow_tasks_find_mut_n(2_500) }
    #[test]
    fn cow_tasks_find_mut_10_000() { cow_tasks_find_mut_n(10_000) }
    #[test]
    fn cow_tasks_find_mut_100_000() { cow_tasks_find_mut_n(100_000) }
}

mod join {
    use cow::btree::{BTreeMap, BTreeSet};
    use cow::join::{join_maps, join_sets, join_set_to_map};

    #[test]
    fn test_map_join_10_shared_set()
    {
        let mut a = BTreeMap::new();
        let mut b = BTreeMap::new();

        for i in range(0, 10) {
            a.insert(i, i);
            b.insert(i, i);
        }

        let mut r_iter = range(0, 10);

        for (k, (data_a, data_b)) in join_maps(a.iter(), b.iter()) {
            let i = r_iter.next().unwrap();
            assert!(i == *k);
            assert!(i == *data_a);
            assert!(i == *data_b);
        }
    }

    #[test]
    fn test_map_join_10_no_shared()
    {
        let mut a = BTreeMap::new();
        let mut b = BTreeMap::new();

        for i in range(0, 10) {
            a.insert(i, i);
        }

        for i in range(10, 20) {
            b.insert(i, i);
        }

        for _ in join_maps(a.iter(), b.iter()) {
            fail!("should not have found any shared items!");
        }
    }

    #[test]
    fn test_map_join_10_even()
    {
        let mut a = BTreeMap::new();
        let mut b = BTreeMap::new();

        for i in range(0, 10) {
            a.insert(i, i);
        }

        for i in range(0, 10) {
            if i % 2 == 0 {
                b.insert(i, i);
            }
        }

        for (k, _) in join_maps(a.iter(), b.iter()) {
            assert!(k % 2 == 0);
        }
    }

    #[test]
    fn test_set_join_10_shared_set()
    {
        let mut a = BTreeSet::new();
        let mut b = BTreeSet::new();

        for i in range(0, 10) {
            a.insert(i);
            b.insert(i);
        }

        let mut r_iter = range(0, 10);

        for k in join_sets(a.iter(), b.iter()) {
            let i = r_iter.next().unwrap();
            assert!(i == *k);
        }
    }

    #[test]
    fn test_set_join_10_no_shared()
    {
        let mut a = BTreeSet::new();
        let mut b = BTreeSet::new();

        for i in range(0, 10) {
            a.insert(i);
        }

        for i in range(10, 20) {
            b.insert(i);
        }

        for _ in join_sets(a.iter(), b.iter()) {
            fail!("should not have found any shared items!");
        }
    }

    #[test]
    fn test_set_join_10_even()
    {
        let mut a = BTreeSet::new();
        let mut b = BTreeSet::new();

        for i in range(0, 10) {
            a.insert(i);
        }

        for i in range(0, 10) {
            if i % 2 == 0 {
                b.insert(i);
            }
        }

        for k in join_sets(a.iter(), b.iter()) {
            assert!(k % 2 == 0);
        }
    }

    #[test]
    fn test_map_and_set_join_10_shared_set()
    {
        let mut a = BTreeSet::new();
        let mut b = BTreeMap::new();

        for i in range(0, 10) {
            a.insert(i);
            b.insert(i, i);
        }

        let mut r_iter = range(0, 10);

        for (k, _) in join_set_to_map(a.iter(), b.iter()) {
            let i = r_iter.next().unwrap();
            assert!(i == *k);
        }
    }

    #[test]
    fn test_map_and_set_join_10_no_shared()
    {
        let mut a = BTreeSet::new();
        let mut b = BTreeMap::new();

        for i in range(0, 10) {
            a.insert(i);
        }

        for i in range(10, 20) {
            b.insert(i, i);
        }

        for _ in join_set_to_map(a.iter(), b.iter()) {
            fail!("should not have found any shared items!");
        }
    }

    #[test]
    fn test_map_and_set_join_10_even()
    {
        let mut a = BTreeSet::new();
        let mut b = BTreeMap::new();

        for i in range(0, 10) {
            a.insert(i);
        }

        for i in range(0, 10) {
            if i % 2 == 0 {
                b.insert(i, i);
            }
        }

        for (k, _) in join_set_to_map(a.iter(), b.iter()) {
            assert!(k % 2 == 0);
        }
    }

    #[test]
    fn test_complex()
    {
        let mut a = BTreeSet::new();
        let mut b = BTreeSet::new();
        let mut c = BTreeMap::new();
        let mut d = BTreeMap::new();

        for i in range(0, 100) {
            a.insert(i);
            if i % 2 == 0 {
                b.insert(i);
            }
            if i % 3 == 0 {
                c.insert(i, i);
            }
            if i % 5 == 0 {
                d.insert(i, i);
            }
        }

        for (k, _) in join_set_to_map(join_sets(a.iter(), b.iter()), join_maps(c.iter(), d.iter()))
        {
            assert!(k % 30 == 0);
        }

    }
}