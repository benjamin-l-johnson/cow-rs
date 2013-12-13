
extern mod extra;

use std::kinds::{Freeze, Send};
use std::num::{zero, Zero};
use std::util;

use extra::arc::Arc;

static LEAF_SIZE: uint = 31;
static INTERNAL_SIZE: uint = 16;

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
    Leaf(~NodeLeaf<K, V>),
    SharedInternal(Arc<~NodeInternal<K, V>>),
    SharedLeaf(Arc<~NodeLeaf<K, V>>)
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

pub struct BTreeStat {
    mut_leafs: uint,
    mut_nodes: uint,
    immut_leafs: uint,
    immut_nodes: uint,
    items: uint,
    unused: uint,
    depth: uint
}

impl Zero for BTreeStat
{
    fn zero() -> BTreeStat
    {
        BTreeStat {
            mut_leafs: 0u,
            mut_nodes: 0u,
            immut_leafs: 0u,
            immut_nodes: 0u,
            items: 0u,
            unused: 0u,
            depth: 0u
        }
    }

    fn is_zero(&self) -> bool
    {
        self.mut_leafs.is_zero() &&
        self.mut_nodes.is_zero() &&
        self.immut_leafs.is_zero() &&
        self.immut_nodes.is_zero() &&
        self.items.is_zero() &&
        self.unused.is_zero()
    }
}

impl Add<BTreeStat, BTreeStat> for BTreeStat
{
    fn add(&self, other: &BTreeStat) -> BTreeStat
    {
        BTreeStat {
            mut_leafs: self.mut_leafs + other.mut_leafs,
            mut_nodes: self.mut_nodes + other.mut_nodes,
            immut_leafs: self.immut_leafs + other.immut_leafs,
            immut_nodes: self.immut_nodes + other.immut_nodes,
            items: self.items + other.items,
            unused: self.unused + other.unused,
            depth: self.depth.max(&other.depth)
        }
    }
}

impl<K: Zero+Clone+Ord+Eq+Send+Freeze, V: Zero+Clone+Send+Freeze> Clone for Node<K, V>
{
    fn clone(&self) -> Node<K, V>
    {
        match *self {
            Empty => Empty,
            Internal(ref node) => Internal(node.clone()),
            Leaf(ref leaf) => Leaf(leaf.clone()), 
            SharedInternal(ref node) => SharedInternal(node.clone()),
            SharedLeaf(ref leaf) => SharedLeaf(leaf.clone()), 
        }
    }
}

impl<K: Zero+Clone+Ord+Eq+Send+Freeze, V: Zero+Clone+Send+Freeze> Node<K, V>
{
    //#[inline(always)]
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
            },
            SharedLeaf(_) => {
                Unfreeze
            },
            SharedInternal(_) => {
                Unfreeze
            },
        }
    }

    //#[inline(always)]
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
            },
            SharedLeaf(ref leaf) => {
                (*leaf).get().get(key)
            },
            SharedInternal(ref node) => {
                (*node).get().get(key)
            },
        }
    }

    //#[inline(always)]
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

    //#[inline(always)]
    pub fn freeze(&mut self)
    {
        match *self {
            Empty | SharedLeaf(_) | SharedInternal(_) => return,
            Internal(_) | Leaf(_) => ()
        };

        let mut old = Empty;
        util::swap(&mut old, self);

        *self = match old {
            Internal(mut node) => {
                node.freeze_children();
                SharedInternal(Arc::new(node))
            },
            Leaf(leaf) => {
                SharedLeaf(Arc::new(leaf))
            },
            _ => {fail!("This should not have been reached")}
        };
    }

    //#[inline(always)]
    pub fn unfreeze(&mut self)
    {
        match *self {
            Empty | Internal(_) | Leaf(_) => return,
            SharedLeaf(_) | SharedInternal(_) => ()
        };

        let mut old = Empty;
        util::swap(&mut old, self);

        *self = match old {
            SharedInternal(node) => {
                Internal(node.get().clone())
            },
            SharedLeaf(leaf) => {
                Leaf(leaf.get().clone())
            },
            _ => {old}
        };
    }

    pub fn stat(&self) -> BTreeStat
    {
        let mut stat = match *self {
            Empty => {
                zero()
            },
            Leaf(ref leaf) => {
                let mut stat = (*leaf).stat();
                stat.mut_leafs += 1;
                stat
            },
            Internal(ref node) => {
                let mut stat = (*node).stat();
                stat.mut_nodes += 1;
                stat
            },
            SharedLeaf(ref leaf) => {
                let mut stat = (*leaf).get().stat();
                stat.immut_leafs += 1;
                stat
            },
            SharedInternal(ref node) => {
                let mut stat = (*node).get().stat();
                stat.immut_nodes += 1;
                stat
            },
        };

        stat.depth += 1;
        stat
    }
}

impl<K: Zero+Clone+Ord+Eq+Send+Freeze, V: Zero+Clone+Send+Freeze> Clone for BTree<K, V>
{
    fn clone(&self) -> BTree<K, V>
    {
        BTree {
            root: self.root.clone()
        }    
    }
}

impl<K: Zero+Clone+Ord+Eq+Send+Freeze, V: Zero+Clone+Send+Freeze> BTree<K, V>
{
    pub fn new() -> BTree<K, V>
    {
        println!("leaf: {} node: {}",
                std::mem::size_of::<NodeLeaf<K, V>>(),
                std::mem::size_of::<NodeInternal<K, V>>());
        BTree {
            root: Empty
        }
    }

    pub fn set(&mut self, key: &K, value: &V)
    {
        match self.root.set(key, value) {
            NoAction => (),
            UpdateLeft(_) => (),
            Unfreeze => {
                self.root.unfreeze();
                self.set(key, value);
            }
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
        }
    }

    pub fn get<'a>(&'a self, key: &K) -> Option<&'a V>
    {
        self.root.get(key)
    }

    pub fn freeze(&mut self)
    {
        self.root.freeze()
    }

    pub fn unfreeze(&mut self)
    {
        self.root.unfreeze()
    }

    pub fn stat(&self) -> BTreeStat
    {
        self.root.stat()
    }
}


impl<K: Zero+Clone+Ord+Eq+Send+Freeze, V: Zero+Clone+Send+Freeze> Clone for NodeLeaf<K, V>
{
    fn clone(&self) -> NodeLeaf<K, V>
    {
        let mut new = NodeLeaf::new();

        new.used = self.used;

        for i in range(0, self.used) {
            new.values[i] = self.values[i].clone();
            new.keys[i] = self.keys[i].clone();
        }

        new
    }
}

impl<K: Zero+Clone+Ord+Eq+Send+Freeze, V: Zero+Clone+Send+Freeze> NodeLeaf<K, V>
{
    fn new() -> NodeLeaf<K, V>
    {
        NodeLeaf {
            used: 0,
            keys: [zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                   zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                   zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                   zero(), zero(), zero(), zero(), zero(), zero(), zero()],
            values: [zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                     zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                     zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                     zero(), zero(), zero(), zero(), zero(), zero(), zero()]
        }
    }

    //#[inline(always)]
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

    //#[inline(always)]
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

    fn stat(&self) -> BTreeStat
    {
        let mut stat: BTreeStat = zero();

        stat.items = self.used;
        stat.unused = LEAF_SIZE - self.used;

        stat
    }
}

impl<K: Zero+Clone+Ord+Eq+Send+Freeze, V: Zero+Clone+Send+Freeze> Clone for NodeInternal<K, V>
{
    fn clone(&self) -> NodeInternal<K, V>
    {
        let mut new = NodeInternal::new_empty();

        for i in range(0, self.used) {
            new.children[i] = self.children[i].clone();
        }

        for i in range(0, self.used-1) {
            new.keys[i] = self.keys[i].clone();
        }

        new.used = self.used;

        new
    }
}

impl<K: Zero+Clone+Ord+Eq+Send+Freeze, V: Zero+Clone+Send+Freeze> NodeInternal<K, V>
{
    fn new(key: K, left: Node<K, V>, right: Node<K, V>) -> NodeInternal<K, V>
    {
        NodeInternal {
            used: 2,
            keys: [key,    zero(), zero(), zero(), zero(), zero(), zero(),zero(),
                   zero(), zero(), zero(), zero(), zero(), zero(), zero()]
            ,
            children: [left,  right, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty]
        }
    }

    fn new_empty() -> NodeInternal<K, V>
    {
        NodeInternal {
            used: 0,
            keys: [zero(), zero(), zero(), zero(), zero(), zero(), zero(), zero(),
                   zero(), zero(), zero(), zero(), zero(), zero(), zero()],
            children: [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty]
        }
    }

    //#[inline(always)]
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
            Unfreeze => {
                self.children[idx].unfreeze();
                self.set(key, value)
            }
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
                    NoAction
                }
            } 
        }
    }

    //#[inline(always)]
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

    fn freeze_children(&mut self)
    {
        for i in range(0, self.used) {
            self.children[i].freeze();
        }
    }

    fn stat(&self) -> BTreeStat
    {
        let mut stat: BTreeStat = zero();
        for i in range(0, self.used) {
            stat = stat.add(&self.children[i].stat());
        }
        stat
    }
}
