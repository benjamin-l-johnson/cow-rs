
extern mod extra;

use std::kinds::{Freeze, Send};
use std::default::{Default};
use std::util;
use std::num::{Zero, zero};
use extra::arc::Arc;

static LEAF_SIZE: uint = 31;
static INTERNAL_SIZE: uint = 128;

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

enum InsertAction<K> {
    InsertDone(bool),
    Split,
    InsertUpdateLeft(K),
    InsertUnfreeze
}

enum RemoveAction<K> {
    RemoveDone(bool, bool),
    RemoveUpdateLeft(K),
    RemoveUnfreeze
}

pub struct BTreeStat {
    mut_leaves: uint,
    mut_nodes: uint,
    immut_leaves: uint,
    immut_nodes: uint,
    items: uint,
    unused: uint,
    depth: uint
}

fn default<T: Default>() -> T
{
    Default::default()
}

impl Zero for BTreeStat
{
    fn zero() -> BTreeStat
    {
        BTreeStat {
            mut_leaves: 0u,
            mut_nodes: 0u,
            immut_leaves: 0u,
            immut_nodes: 0u,
            items: 0u,
            unused: 0u,
            depth: 0u
        }
    }

    fn is_zero(&self) -> bool
    {
        self.mut_leaves.is_zero() &&
        self.mut_nodes.is_zero() &&
        self.immut_leaves.is_zero() &&
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
            mut_leaves: self.mut_leaves + other.mut_leaves,
            mut_nodes: self.mut_nodes + other.mut_nodes,
            immut_leaves: self.immut_leaves + other.immut_leaves,
            immut_nodes: self.immut_nodes + other.immut_nodes,
            items: self.items + other.items,
            unused: self.unused + other.unused,
            depth: self.depth.max(&other.depth)
        }
    }
}

impl<K: Default+Clone+Ord+Eq+Send+Freeze, V: Default+Clone+Send+Freeze> Clone for Node<K, V>
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

impl<K: Default+Clone+Ord+Eq+Send+Freeze, V: Default+Clone+Send+Freeze> Node<K, V>
{
    pub fn insert(&mut self, key: &K, value: &V) -> InsertAction<K>
    {
        match *self {
            Empty => {
                *self = Leaf(~NodeLeaf::new());
                self.insert(key, value)
            },
            Leaf(ref mut leaf) => {
                (*leaf).insert(key, value)
            },
            Internal(ref mut node) => {
                (*node).insert(key, value)
            },
            SharedLeaf(_) => {
                InsertUnfreeze
            },
            SharedInternal(_) => {
                InsertUnfreeze
            },
        }
    }

    pub fn remove(&mut self, key: &K) -> RemoveAction<K>
    {
        match *self {
            Empty => {
                RemoveDone(false, false)
            },
            Leaf(ref mut leaf) => {
                (*leaf).remove(key)
            },
            Internal(ref mut node) => {
                (*node).remove(key)
            },
            SharedLeaf(_) => {
                RemoveUnfreeze
            },
            SharedInternal(_) => {
                RemoveUnfreeze
            },
        }        
    }

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
                stat.mut_leaves += 1;
                stat
            },
            Internal(ref node) => {
                let mut stat = (*node).stat();
                stat.mut_nodes += 1;
                stat
            },
            SharedLeaf(ref leaf) => {
                let mut stat = (*leaf).get().stat();
                stat.immut_leaves += 1;
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

impl<K: Default+Clone+Ord+Eq+Send+Freeze, V: Default+Clone+Send+Freeze> Clone for BTree<K, V>
{
    fn clone(&self) -> BTree<K, V>
    {
        BTree {
            root: self.root.clone()
        }    
    }
}

impl<K: Default+Clone+Ord+Eq+Send+Freeze, V: Default+Clone+Send+Freeze> BTree<K, V>
{
    pub fn new() -> BTree<K, V>
    {
        BTree {
            root: Empty
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> bool
    {
        match self.root.insert(&key, &value) {
            InsertDone(update) => update,
            // if the left key is only updated
            // on an insert, not an update so this
            // can return false
            InsertUpdateLeft(_) => false,
            InsertUnfreeze => {
                self.root.unfreeze();
                self.insert(key, value)
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
                self.insert(key, value)
            }
        }
    }

    pub fn remove(&mut self, key: &K) -> bool
    {
        match self.root.remove(key) {
            RemoveDone(found, _) => found,
            _ => fail!("not supported")
        }
    }

    pub fn get<'a>(&'a self, key: &K) -> Option<&'a V>
    {
        let mut target = &self.root;
        let mut target_leaf: Option<&~NodeLeaf<K, V>> = None;

        while target_leaf.is_none() {
            match *target {
                SharedInternal(ref node) => {
                    let node = (*node).get();
                    target = &node.children[node.search(key)];
                },
                Internal(ref node) => {
                    target = &(*node).children[(*node).search(key)];
                },
                SharedLeaf(ref leaf) => {
                    target_leaf = Some(leaf.get());
                },
                Leaf(ref leaf) => {
                    target_leaf = Some(leaf);
                },

                Empty => {
                    return None;
                },
            };
        }

        target_leaf.unwrap().get(key)
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


impl<K: Default+Clone+Ord+Eq+Send+Freeze, V: Default+Clone+Send+Freeze> Clone for NodeLeaf<K, V>
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

impl<K: Default+Clone+Ord+Eq+Send+Freeze, V: Default+Clone+Send+Freeze> NodeLeaf<K, V>
{
    fn new() -> NodeLeaf<K, V>
    {
        NodeLeaf {
            used: 0,
            keys: [default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default()],
            values: [default(), default(), default(), default(), default(), default(), default(), default(),
                     default(), default(), default(), default(), default(), default(), default(), default(),
                     default(), default(), default(), default(), default(), default(), default(), default(),
                     default(), default(), default(), default(), default(), default(), default()]
        }
    }

    fn insert(&mut self, key: &K, value: &V) -> InsertAction<K>
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

            // update
            if insert != self.used && *key == self.keys[insert] {
                self.values[insert] = (*value).clone();
                InsertDone(true)
            // insert
            } else {
                let mut key = (*key).clone();
                let mut value = (*value).clone();

                self.used += 1;

                for j in range(insert, self.used) {
                    util::swap(&mut self.keys[j], &mut key);
                    util::swap(&mut self.values[j], &mut value);
                }

                if insert == self.used {
                    InsertUpdateLeft(self.keys[self.used-1].clone())
                } else {
                    InsertDone(false)
                }
            }
        }
    }

    fn search(&self, key: &K) -> Option<uint>
    {
        for i in range(0, self.used) {
            if *key == self.keys[i] {
                return Some(i);
            }
        }
        None
    }

    fn remove(&mut self, key: &K) -> RemoveAction<K>
    {
        let idx = match self.search(key) {
            Some(idx) => idx,
            None => return RemoveDone(false, false)
        };

        let mut key = default();
        let mut value = default();

        let offset = self.used - 1 + idx;
        for i in range(idx, self.used) {
            let i = offset - i;
            util::swap(&mut self.keys[i], &mut key);
            util::swap(&mut self.values[i], &mut value);
        }

        self.used -= 1;

        RemoveDone(true, self.used < LEAF_SIZE / 2)
    }



    fn get<'a>(&'a self, key: &K) -> Option<&'a V>
    {
        match self.search(key) {
            Some(idx) => Some(&self.values[idx]),
            None => None
        }
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

impl<K: Default+Clone+Ord+Eq+Send+Freeze, V: Default+Clone+Send+Freeze> Clone for NodeInternal<K, V>
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

impl<K: Default+Clone+Ord+Eq+Send+Freeze, V: Default+Clone+Send+Freeze> NodeInternal<K, V>
{
    fn new(key: K, left: Node<K, V>, right: Node<K, V>) -> NodeInternal<K, V>
    {
        NodeInternal {
            used: 2,
            keys: [key,       default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default()]
            ,
            children: [left,  right, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty]
        }
    }

    fn new_empty() -> NodeInternal<K, V>
    {
        NodeInternal {
            used: 0,
            keys: [default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default()],
            children: [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                       Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty]
        }
    }

    fn insert(&mut self, key: &K, value: &V) -> InsertAction<K>
    {
        let idx = self.search(key);

        match self.children[idx].insert(key, value) {
            InsertDone(bool) => (InsertDone(bool)),
            InsertUnfreeze => {
                self.children[idx].unfreeze();
                self.insert(key, value)
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
                    self.insert(key, value)
                }
            },
            InsertUpdateLeft(left) => {
                if idx != self.used {
                    self.keys[idx] = left;
                    InsertUpdateLeft(self.keys[self.used-2].clone())
                } else {
                    InsertDone(false)
                }
            } 
        }
    }

    fn remove(&mut self, key: &K) -> RemoveAction<K>
    {
        self.children[self.search(key)].remove(key)
    }

    fn search_linear(&self, key: &K) -> uint
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

    fn search_bsec(&self, key: &K) -> uint
    {
        let mut start = 0u;
        let mut end = self.used-2;

        while end > start {
            let mid = start + ((end-start) / 2);

            if *key > self.keys[mid] {
                start = mid+1;
            } else {
                end = mid;
            }
        }

        if self.used - 2 == start && *key > self.keys[start] {
            start + 1
        } else {
            start
        }
    }

    fn search(&self, key: &K) -> uint
    {
        if self.used < 16 {
            self.search_linear(key)
        } else {
            self.search_bsec(key)
        }
    }

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

        let mut key: K = default();
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