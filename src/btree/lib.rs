
extern mod extra;

use std::num::{zero, Zero};
use std::util;

//use extra::arc::Arc;

static LEAF_SIZE: uint = 63;
static INTERNAL_SIZE: uint = 32;

struct NodeLeaf<K, V> {
    used:   uint,
    keys:   [K, ..LEAF_SIZE],
    values: [V, ..LEAF_SIZE] 
}

struct NodeInternal<K, V> {
    used:     uint,
    keys:     [K, ..INTERNAL_SIZE-1],
    children: [Node<K, V>, ..INTERNAL_SIZE]
}

enum Node<K, V> {
    Empty,
    Internal(~NodeInternal<K, V>),
    Leaf(~NodeLeaf<K, V>)
}

pub struct BTree<K, V> {
    root: Node<K, V>
}

enum ActionNeeded<K> {
    NoAction,
    Split,
    UpdateLeft(K),
    Unfreeze
}

impl<K: Zero+Clone+Ord+Eq, V: Zero+Clone> Node<K, V>
{

    #[inline(always)]
    pub fn set(&mut self, key: &K, value: &V) -> ActionNeeded<K>
    {
        match *self {
            Empty => {
                *self = Leaf(~NodeLeaf::new());
                self.set(key, value);
                NoAction
            },
            Leaf(ref mut leaf) => {
                (*leaf).set(key, value)
            },
            Internal(ref mut node) => {
                (*node).set(key, value)
            }
        }
    }

    #[inline(always)]
    pub fn get<'a>(&'a self, key: &K) -> Option<&'a V>
    {
        match *self {
            Empty => {
                None
            },
            Leaf(ref leaf) => {
                (*leaf).get(key)
            },
            Internal(ref node) => {
                (*node).get(key)
            }
        }
    }

    #[inline(always)]
    pub fn split(&mut self) -> (Node<K, V>, K)
    {
        match *self {
            Leaf(ref mut leaf) => {
                let (leaf, key) = (*leaf).split();
                (Leaf(~leaf), key)
            },
            Internal(ref mut node) => {
                let (node, key) = (*node).split();
                (Internal(~node), key)
            },
            _ => {
                fail!("unsupported split");
            }
        }
    }

    pub fn print(&self)
    {
        println(format!("self.nodes {:?}", self));
    }
}


impl<K: Zero+Clone+Ord+Eq, V: Zero+Clone> BTree<K, V>
{
    pub fn new() -> BTree<K, V>
    {
        BTree {
            root: Empty
        }

    }

    pub fn set(&mut self, key: &K, value: &V)
    {
        match self.root.set(key, value) {
            NoAction => (),
            UpdateLeft(_) => (),
            Split => {
                let (split_key, right) = match self.root {
                    Leaf(ref mut leaf) => {
                        let (right, key) = leaf.split();
                        (key, Leaf(~right))
                    },
                    Internal(ref mut node) => {
                        let (right, key) = node.split();
                        (key, Internal(~right))

                    }
                    _ => fail!("this is impossible")
                };
                let mut left = Empty;

                util::swap(&mut self.root, &mut left);

                self.root = Internal(~NodeInternal::new(split_key, left, right));
                self.set(key, value);
            }
            Unfreeze => fail!("no supported!!!")
        }
    }

    pub fn get<'a>(&'a self, key: &K) -> Option<&'a V>
    {
        self.root.get(key)
    }

    pub fn print(&self)
    {
        self.root.print();
    }
}

impl<K: Zero+Clone+Ord+Eq, V: Zero+Clone> NodeLeaf<K, V>
{
    fn new() -> NodeLeaf<K, V>
    {
        NodeLeaf {
            used: 0,
            keys: [
                zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                zero(), zero(), zero(), zero(), zero(), zero(), zero()
            ],
            values: [
                zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                zero(), zero(), zero(), zero(), zero(), zero(), zero()
            ]
        }
    }

    #[inline(always)]
    fn set(&mut self, key: &K, value: &V) -> ActionNeeded<K>
    {
        if self.used == LEAF_SIZE {
            Split
        } else {
            let mut insert = 0;
            while insert < self.used {
                if *key <= self.keys[insert] {
                    break;
                }
                insert += 1;
            }

            let mut key = (*key).clone();
            let mut value = (*value).clone();

            self.used += 1;

            for j in range(insert, self.used) {
                util::swap(&mut self.keys[j], &mut key);
                util::swap(&mut self.values[j], &mut value);
            }

            if insert == self.used {
                UpdateLeft(self.keys[self.used-1].clone())
            } else {
                NoAction
            }
        }
    }

    #[inline(always)]
    fn get<'a>(&'a self, key: &K) -> Option<&'a V>
    {
        for i in range(0, self.used) {
            if *key == self.keys[i] {
                return Some(&self.values[i]);
            }
        }
        None
    }

    fn split(&mut self) -> (NodeLeaf<K, V>, K)
    {
        let mut right = NodeLeaf::new();

        for (dst, src) in range(LEAF_SIZE / 2, self.used).enumerate() {
            util::swap(&mut right.keys[dst], &mut self.keys[src]);
            util::swap(&mut right.values[dst], &mut self.values[src]);
        }

        right.used = self.used - LEAF_SIZE / 2;
        self.used =  LEAF_SIZE / 2;

        (right, self.keys[self.used-1].clone())
    }
}

impl<K: Zero+Clone+Ord+Eq, V: Zero+Clone> NodeInternal<K, V>
{
    fn new(key: K, left: Node<K, V>, right: Node<K, V>) -> NodeInternal<K, V>
    {
        NodeInternal {
            used: 2,
            keys: [
                key,    zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                zero(), zero(), zero(), zero(), zero(), zero(), zero()
            ],
            children: [
                left,  right, Empty, Empty, Empty, Empty, Empty, Empty,
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
            ]
        }
    }

    fn new_empty() -> NodeInternal<K, V>
    {
        NodeInternal {
            used: 0,
            keys: [
                zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                zero(), zero(), zero(), zero(), zero(), zero(), zero()
            ],
            children: [
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
            ]
        }
    }

    #[inline(always)]
    fn set(&mut self, key: &K, value: &V) -> ActionNeeded<K>
    {
        let mut idx = 0;
        while idx < (self.used-1) {
            if *key < self.keys[idx] {
                break;
            }
            idx += 1;
        }

        match self.children[idx].set(key, value) {
            NoAction => (NoAction),
            Split => {
                if self.used == INTERNAL_SIZE {
                    Split
                } else {
                    let (right, split_key) = self.children[idx].split();
                    let mut right = right;
                    let mut split_key = split_key;

                    let idx = idx + 1;

                    // move items left
                    for j in range(idx-1, self.used) {
                        util::swap(&mut self.keys[j], &mut split_key);
                    }

                    for j in range(idx, self.used+1) {
                        util::swap(&mut self.children[j], &mut right);
                    }

                    self.used += 1;
                    self.set(key, value)
                }
            },
            UpdateLeft(left) => {
                assert!(left != zero());
                if idx != self.used {
                    self.keys[idx] = left;
                    UpdateLeft(self.keys[self.used-2].clone())
                } else {
                    UpdateLeft(left)
                }
            } 
            Unfreeze => fail!("not supported")
        }
    }

    #[inline(always)]
    fn search(&self, key: &K) -> uint
    {
        let mut idx = 0;
        while idx < (self.used-1) {
            if *key <= self.keys[idx] {
                break;
            }
            idx += 1;
        }
        idx
    }

    #[inline(always)]
    fn get<'a>(&'a self, key: &K) -> Option<&'a V>
    {
        self.children[self.search(key)].get(key)
    }

    fn split(&mut self) -> (NodeInternal<K, V>, K)
    {
        let mut right = NodeInternal::new_empty();

        for (dst, src) in range(INTERNAL_SIZE / 2, self.used).enumerate() {
            util::swap(&mut right.children[dst], &mut self.children[src]);
        }

        for (dst, src) in range(INTERNAL_SIZE / 2, self.used-1).enumerate() {
            util::swap(&mut right.keys[dst], &mut self.keys[src]);
        }

        right.used = self.used - INTERNAL_SIZE / 2;
        self.used =  INTERNAL_SIZE / 2;

        let mut key: K = zero();
        util::swap(&mut key, &mut self.keys[self.used-1]);

        (right, key)
    }

}
