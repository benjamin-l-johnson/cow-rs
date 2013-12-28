
#[pkgid = "github.com/csherratt/arc-btree-rs#btree:0.1"];

extern mod extra;

use std::kinds::{Freeze, Send};
use std::default::{Default};
use std::util;
use std::sync::atomics::{AtomicUint, SeqCst, Acquire};
use std::cast::{transmute, transmute_mut};

static LEAF_SIZE: uint = 63;
static INTERNAL_SIZE: uint = 128;

struct NodeLeaf<K, V> {
    ref_count: AtomicUint,
    used:   uint,
    keys:   [K, ..LEAF_SIZE],
    values: [V, ..LEAF_SIZE] 
}

struct NodeInternal<K, V> {
    ref_count: AtomicUint,
    used:      uint,
    keys:      [K, ..INTERNAL_SIZE-1],
    children:  [Node<K, V>, ..INTERNAL_SIZE]
}

enum Node<K, V> {
    Empty,
    Internal(*mut NodeInternal<K, V>),
    Leaf(*mut NodeLeaf<K, V>),
}

pub struct BTree<K, V> {
    root: Node<K, V>
}

enum InsertAction<K> {
    InsertDone(bool),
    Split,
    InsertUpdateLeft(K)
}

fn default<T: Default>() -> T
{
    Default::default()
}

impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> Clone for Node<K, V>
{
    fn clone(&self) -> Node<K, V>
    {
        unsafe {
            match *self {
                Empty => Empty,
                Internal(node) => {
                    let tmp: &mut NodeInternal<K, V> = transmute(node);
                    tmp.inc_ref();
                    Internal(node)
                },
                Leaf(leaf) => {
                    let tmp: &mut NodeLeaf<K, V> = transmute(leaf);
                    tmp.inc_ref();
                    Leaf(leaf)
                }
            }
        }
    }
}

impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> Node<K, V>
{
    unsafe fn insert(&mut self, key: &K, value: &V) -> InsertAction<K>
    {
        self.mutable();

        match *self {
            Empty => {
                let leaf: ~NodeLeaf<K, V> = ~NodeLeaf::new();
                *self = Leaf(transmute(leaf));
                self.insert(key, value)
            },
            Leaf(ref mut leaf) => {
                let leaf: &mut NodeLeaf<K, V> = transmute(*leaf);
                leaf.insert(key, value)
            },
            Internal(ref mut node) => {
                let node: &mut NodeInternal<K, V> = transmute(*node);
                node.insert(key, value)
            }
        }
    }

    unsafe fn pop(&mut self, key: &K) -> (Option<K>, Option<V>, bool)
    {
        self.mutable();

        match *self {
            Empty => {
                (None, None, false)
            },
            Leaf(ref mut leaf) => {
                let leaf: &mut NodeLeaf<K, V> = transmute(*leaf);
                leaf.pop(key)
            },
            Internal(ref mut node) => {
                let node: &mut NodeInternal<K, V> = transmute(*node);
                node.pop(key)
            }
        }        
    }

    unsafe fn split(&mut self) -> (Node<K, V>, K)
    {
        match *self {
            Leaf(ref mut leaf) => {
                let leaf: &mut NodeLeaf<K, V> = transmute(*leaf);
                let (leaf, key) = leaf.split();
                (Leaf(transmute(~leaf)), key)
            },
            Internal(ref mut node) => {
                let node: &mut NodeInternal<K, V> = transmute(*node);
                let (node, key) = node.split();
                (Internal(transmute(~node)), key)
            },
            _ => {
                fail!("unsupported split");
            }
        }
    }

    unsafe fn len(&self) -> uint
    {
        match *self {
            Empty => {
                0u
            },
            Leaf(ref leaf) => {
                let leaf: &NodeLeaf<K, V> = transmute(*leaf);
                leaf.len()
            },
            Internal(ref node) => {
                let node: &NodeInternal<K, V> = transmute(*node);
                node.len()
            }
        }     
    }

    unsafe fn find_mut<'a>(&'a mut self, key: &K) -> Option<&'a mut V>
    {
        self.mutable();

        match *self {
            Empty => {
                None
            },
            Leaf(ref mut leaf) => {
                let leaf: &mut NodeLeaf<K, V> = transmute(*leaf);
                leaf.find_mut(key)
            },
            Internal(ref mut node) => {
                let node: &mut NodeInternal<K, V> = transmute(*node);
                node.find_mut(key)
            },
        }
    }

    // move the lowest key from other to self iff node is has extra keys
    unsafe fn rotate_right(&mut self, src: &mut Node<K, V>) -> bool
    {
        src.mutable();

        match (self, src) {
            (&Leaf(ref mut sink), &Leaf(ref mut src)) => {
                let src: &mut NodeLeaf<K, V> = transmute(*src);
                let sink: &mut NodeLeaf<K, V> = transmute(*sink);
                sink.rotate_right(src)
            },
            (&Internal(ref mut sink), &Internal(ref mut src)) => {
                let src: &mut NodeInternal<K, V> = transmute(*src);
                let sink: &mut NodeInternal<K, V> = transmute(*sink);
                sink.rotate_right(src)
            },
            (_, _) => {
                fail!("both nodes should be of the same type");
            }
        }
    }

    // move highest key from src to self iff node is has extra keys
    unsafe fn rotate_left(&mut self, src: &mut Node<K, V>) -> bool
    {
        src.mutable();

        match (self, src) {
            (&Leaf(ref mut sink), &Leaf(ref mut src)) => {
                let src: &mut NodeLeaf<K, V> = transmute(*src);
                let sink: &mut NodeLeaf<K, V> = transmute(*sink);
                sink.rotate_left(src)
            },
            (&Internal(ref mut sink), &Internal(ref mut src)) => {
                let src: &mut NodeInternal<K, V> = transmute(*src);
                let sink: &mut NodeInternal<K, V> = transmute(*sink);
                sink.rotate_left(src)
            },
            (_, _) => {
                fail!("both nodes should be of the same type");
            }
        }
    }

    // move highest key from src to self iff node is has extra keys
    unsafe fn merge(&mut self, src: Node<K, V>)
    {
        let mut src = src;
        src.mutable();
        match (self, src) {
            (&Leaf(ref mut sink), Leaf(ref mut src)) => {
                let tmp: &mut NodeLeaf<K, V> = transmute(*src);
                let sink: &mut NodeLeaf<K, V> = transmute(*sink);
                sink.merge(tmp);
            },
            (&Internal(ref mut sink), Internal(ref mut src)) => {
                let tmp: &mut NodeInternal<K, V> = transmute(*src);
                let sink: &mut NodeInternal<K, V> = transmute(*sink);
                sink.merge(tmp);
            },
            (_, _) => {
                fail!("both nodes should be of the same type");
            }
        }
    }

    unsafe fn max_key(&self) -> K
    {
        match *self {
            Leaf(ref leaf) => {
                let leaf: &NodeLeaf<K, V> = transmute(*leaf);
                leaf.max_key()
            },
            Internal(ref node) => {
                let node: &NodeInternal<K, V> = transmute(*node);
                node.max_key()
            },
            Empty => fail!("invalid node")
        }
    }

    unsafe fn lift(&mut self)
    {
        let depleted = match *self {
            Internal(ref node) => {
                let node: &NodeInternal<K, V> = transmute(*node);
                node.used == 1
            },
            Leaf(ref leaf) => {
                let leaf: &NodeLeaf<K, V> = transmute(*leaf);
                leaf.used == 0
            },
            _ => false
        };

        if depleted {
            let mut child = Empty;
            match *self {
                Internal(ref mut node) => {
                    let tmp: &mut NodeInternal<K, V> = transmute(*node);
                    util::swap(&mut child, &mut tmp.children[0]);
                    let _: ~NodeInternal<K, V> = transmute(*node);

                },
                Leaf(ref node) => {
                    let _: ~NodeLeaf<K, V> = transmute(*node);
                },
                _ => fail!("invalid node")
            }
            *self = child;
        }
    }

    unsafe fn mutable(&mut self)
    {
        let dup = match *self {
            Empty => true,
            Leaf(ref mut leaf) => {
                let leaf: &NodeLeaf<K, V> = transmute(*leaf);
                leaf.mutable()
            },
            Internal(ref mut node) => {
                let node: &NodeInternal<K, V> = transmute(*node);
                node.mutable()
            }
        };

        if !dup {
            let mut new = match *self {
                Empty => Empty,
                Leaf(leaf) => {
                    Leaf(transmute(~(*leaf).clone()))
                },
                Internal(node) => {
                    Internal(transmute(~(*node).clone()))
                }
            };
            util::swap(&mut new, self);
            new.dec_ref();
        }
    }
}

impl<K, V> Node<K, V>
{
    #[inline(always)]
    unsafe fn inc_ref(&self)
    {
        match *self {
            Leaf(ref leaf) => {
                let leaf: &mut NodeLeaf<K, V> = transmute(*leaf);
                leaf.inc_ref()
            },
            Internal(ref node) => {
                let node: &mut NodeInternal<K, V> = transmute(*node);
                node.inc_ref()
            }
            Empty => ()
        }
    }

    #[inline(always)]
    unsafe fn dec_ref(&self)
    {
        match *self {
            Leaf(ref leaf) => {
                let leaf: &mut NodeLeaf<K, V> = transmute(*leaf);
                // last to have access to the node
                if leaf.dec_ref() {
                    let _: ~NodeLeaf<K, V> = transmute(leaf);
                }
            },
            Internal(ref node) => {
                let node: &mut NodeInternal<K, V> = transmute(*node);
                // last to have access to the node
                if node.dec_ref() {
                    let _: ~NodeInternal<K, V> = transmute(node);
                }
            }
            Empty => ()
        }
    }
}

impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> NodeInternal<K, V>
{
    fn new(key: K, left: Node<K, V>, right: Node<K, V>) -> NodeInternal<K, V>
    {
        NodeInternal {
            ref_count: AtomicUint::new(1),
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
            ref_count: AtomicUint::new(1),
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

    unsafe fn insert(&mut self, key: &K, value: &V) -> InsertAction<K>
    {
        let idx = self.search(key);

        match self.children[idx].insert(key, value) {
            InsertDone(bool) => (InsertDone(bool)),
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

    unsafe fn redist(&mut self, idx: uint)
    {
        if idx + 1 != self.used {
            let (left, right) = self.children.mut_split_at(idx+1);
            if left[idx].rotate_left(&mut right[0]) { 
                self.keys[idx] = left[idx].max_key();
                return;
            }
        }

        if idx != 0 {
            let (left, right) = self.children.mut_split_at(idx);
            if right[0].rotate_right(&mut left[idx-1]) {
                self.keys[idx-1] = left[idx-1].max_key();
                return;
            }
        }

        let insert = if idx != 0 {
            idx - 1
        } else if idx + 1 != self.used {
            idx
        } else {
            return;
        };

        let mut child = Empty;
        util::swap(&mut child, &mut self.children[insert+1]);

        self.children[insert].merge(child);
        self.keys[insert] = self.children[insert].max_key();

        if insert+1 != self.used-1  {
            self.keys[insert+1] = default();
        }

        for i in range(insert+1, self.used-1) {
            self.children.swap(i, i+1);
        }
        for i in range(insert+1, self.used-2) {
            self.keys.swap(i, i+1);
        }
        self.used -= 1;

        child.dec_ref();
    }

    unsafe fn pop(&mut self, key: &K) -> (Option<K>, Option<V>, bool)
    {
        let idx = self.search(key);
        let (key, value, needs_merge) = self.children[idx].pop(key);
        let mut key = key;

        if self.used-1 != idx {
            match key {
                Some(k) => {
                    self.keys[idx] = k;
                    key = None;
                },
                None => ()
            }
        }

        if needs_merge {
            self.redist(idx);
        }

        (key, value, self.used < INTERNAL_SIZE / 2)
    }

    unsafe fn find_mut<'a>(&'a mut self, key: &K) -> Option<&'a mut V>
    {
        self.children[self.search(key)].find_mut(key)
    }

    #[inline(always)]
    unsafe fn search(&self, key: &K) -> uint
    {
        let mut start = 0u;
        let mut end = self.used-1;

        while end > start {
            let mid = start + ((end-start) / 2);

            match key.cmp(&self.keys[mid]) {
                Less => end = mid,
                Equal => return mid,
                Greater => start = mid+1,
            }
        }
        if start != self.used-1 {
            match key.cmp(&self.keys[start]) {
                Less | Equal => start,
                Greater => start+1,
            }
        } else {
            start
        }
    }

    unsafe fn split(&mut self) -> (NodeInternal<K, V>, K)
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

    unsafe fn len(&self) -> uint
    {
        let mut len = 0;
        for i in range(0, self.used) {
            len += self.children[i].len();
        }
        len
    }

    unsafe fn rotate_left(&mut self, left: &mut NodeInternal<K, V>) -> bool
    {
        if left.used > INTERNAL_SIZE / 2 {
            self.keys[self.used-1] = self.children[self.used-1].max_key();
            for i in range(0, left.used-2) {
                left.keys.swap(i, i+1);
            }

            util::swap(&mut self.children[self.used], &mut left.children[0]);
            for i in range(0, left.used-1) {
                left.children.swap(i, i+1);
            }

            left.used -= 1;
            self.used += 1;
            true
        } else {
            false
        }
    }

    unsafe fn rotate_right(&mut self, right: &mut NodeInternal<K, V>) -> bool
    {
        if right.used > INTERNAL_SIZE / 2 {
            let key = right.children[right.used-1].max_key();
            for i in range(0, self.used) {
                let i = self.used - i;
                self.keys.swap(i, i-1);
            }
            self.keys[0] = key;

            for i in range(0, self.used) {
                let i = self.used - i;
                self.children.swap(i, i-1);
            }
            util::swap(&mut self.children[0], &mut right.children[right.used-1]);

            right.used -= 1;
            self.used += 1;
            true
        } else {
            false
        }
    }

    unsafe fn merge(&mut self, right: &mut NodeInternal<K, V>)
    {
        self.keys[self.used-1] = self.children[self.used-1].max_key();
        for (src, dst) in range(self.used, self.used+right.used).enumerate() {
            util::swap(&mut right.keys[src], &mut self.keys[dst]);
            util::swap(&mut right.children[src], &mut self.children[dst]);
        }
        self.used += right.used;
    }

    unsafe fn max_key(&self) -> K
    {
        self.children[self.used-1].max_key()
    }

    unsafe fn iter<'a>(&'a self) -> NodeIterator<'a, K, V>
    {
        NodeIterator {
            idx: 0,
            node: self
        }
    }
}

impl<K, V> NodeInternal<K, V>
{
    #[inline(always)]
    unsafe fn inc_ref(&self)
    {
        let mut_self = transmute_mut(self);
        let old_count = (*mut_self).ref_count.fetch_add(1, Acquire);
        assert!(old_count >= 1);
    }

    #[inline(always)]
    unsafe fn dec_ref(&self) -> bool
    {
        let mut_self = transmute_mut(self);
        let old_count = (*mut_self).ref_count.fetch_sub(1, SeqCst);
        
        if old_count == 1 {
            for i in range(0, mut_self.used) {
                let mut to_drop = Empty;
                util::swap(&mut to_drop, &mut mut_self.children[i]);
                to_drop.dec_ref();
            }
            true
        } else {
            false
        }
    }

    #[inline(always)]
    unsafe fn mutable(&self) -> bool
    {
        (*self).ref_count.load(Acquire) == 1
    }
}

impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> Clone for NodeInternal<K, V>
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


impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> NodeLeaf<K, V>
{
    fn new() -> NodeLeaf<K, V>
    {
        NodeLeaf {
            ref_count: AtomicUint::new(1),
            used: 0,
            keys: [default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default(), default(),
                   default(), default(), default(), default(), default(), default(), default()],
            values: [default(), default(), default(), default(), default(), default(), default(), default(),
                     default(), default(), default(), default(), default(), default(), default(), default(),
                     default(), default(), default(), default(), default(), default(), default(), default(),
                     default(), default(), default(), default(), default(), default(), default(), default(),
                     default(), default(), default(), default(), default(), default(), default(), default(),
                     default(), default(), default(), default(), default(), default(), default(), default(),
                     default(), default(), default(), default(), default(), default(), default(), default(),
                     default(), default(), default(), default(), default(), default(), default()]
        }
    }

    #[inline(always)]
    fn search_key(&self, key: &K) -> (bool, uint)
    {
        let mut start = 0u;
        let mut end = self.used;

        while end > start {
            let mid = start + ((end-start) / 2);

            match key.cmp(&self.keys[mid]) {
                Less => end = mid,
                Equal => return (true, mid),
                Greater => start = mid+1,
            }
        }
        (false, start)
    }


    #[inline(always)]
    fn search(&self, key: &K) -> Option<uint>
    {
        match self.search_key(key) {
            (true, idx) => Some(idx),
            (false, _) => None
        }
    }


    fn insert(&mut self, key: &K, value: &V) -> InsertAction<K>
    {
        if self.used == LEAF_SIZE {
            Split
        } else {
            let (found, insert) = self.search_key(key);

            // update
            if insert != self.used && found {
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

    fn pop(&mut self, key: &K) -> (Option<K>, Option<V>, bool)
    {
        let idx = match self.search(key) {
            Some(idx) => idx,
            None => return (None, None, false)
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

        (   if self.used != 0 {
                Some(self.keys[self.used-1].clone())
            } else {
                None
            },
            Some(value),
            self.used < LEAF_SIZE / 2)
    }



    fn find<'a>(&'a self, key: &K) -> Option<&'a V>
    {
        match self.search(key) {
            Some(idx) => Some(&self.values[idx]),
            None => None
        }
    }

    fn find_mut<'a>(&'a mut self, key: &K) -> Option<&'a mut V>
    {
        match self.search(key) {
            Some(idx) => Some(&mut self.values[idx]),
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

    fn len(&self) -> uint
    {
        return self.used;
    }

    fn rotate_left(&mut self, left: &mut NodeLeaf<K, V>) -> bool
    {
        if left.used > LEAF_SIZE / 2 {
            util::swap(&mut self.keys[self.used], &mut left.keys[0]);
            for i in range(0, left.used-1) {
                left.keys.swap(i, i+1);
            }

            util::swap(&mut self.values[self.used], &mut left.values[0]);
            for i in range(0, left.used-1) {
                left.values.swap(i, i+1);
            }

            left.used -= 1;
            self.used += 1;
            true
        } else {
            false
        }
    }

    fn rotate_right(&mut self, right: &mut NodeLeaf<K, V>) -> bool
    {
        if right.used > LEAF_SIZE / 2 {
            for i in range(0, self.used+1) {
                let i = self.used + 1 - i;
                self.keys.swap(i, i-1);
            }
            util::swap(&mut self.keys[0], &mut right.keys[right.used-1]);

            for i in range(0, self.used+1) {
                let i = self.used + 1 - i;
                self.values.swap(i, i-1);
            }
            util::swap(&mut self.values[0], &mut right.values[right.used-1]);

            right.used -= 1;
            self.used += 1;
            true
        } else {
            false
        }
    }

    fn merge(&mut self, right: &mut NodeLeaf<K, V>)
    {
        for (src, dst) in range(self.used, self.used+right.used).enumerate() {
            util::swap(&mut right.keys[src], &mut self.keys[dst]);
            util::swap(&mut right.values[src], &mut self.values[dst]);
        }
        self.used += right.used;
    }

    fn max_key(&self) -> K
    {
        self.keys[self.used-1].clone()
    }


    fn iter<'a>(&'a self) -> LeafIterator<'a, K, V>
    {
        LeafIterator {
            idx: 0,
            leaf: self
        }
    }
}

impl<K, V> NodeLeaf<K, V>
{
    #[inline(always)]
    unsafe fn inc_ref(&self)
    {
        let mut_self = transmute_mut(self);
        let old_count = (*mut_self).ref_count.fetch_add(1, Acquire);
        assert!(old_count >= 1);
    }

    #[inline(always)]
    unsafe fn dec_ref(&self) -> bool
    {
        let mut_self = transmute_mut(self);
        let old_count = (*mut_self).ref_count.fetch_sub(1, SeqCst);
        old_count == 1
    }

    #[inline(always)]
    unsafe fn mutable(&self) -> bool
    {
        (*self).ref_count.load(Acquire) == 1
    }
}

impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> Clone for NodeLeaf<K, V>
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

#[unsafe_destructor]
impl<K, V> Drop for BTree<K, V>
{
    fn drop(&mut self)
    {
        unsafe {self.root.dec_ref()};
    }
}

impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> Container for BTree<K, V> {
    fn len(&self) -> uint
    {
        unsafe{self.root.len()}
    }
}

impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> Map<K, V> for BTree<K, V> {
    #[inline(always)]
    fn find<'a>(&'a self, key: &K) -> Option<&'a V>
    {
        unsafe {
            let mut target = &self.root;
            let mut target_leaf: Option<&NodeLeaf<K, V>> = None;

            while target_leaf.is_none() {
                match *target {
                    Internal(ref node) => {
                        let node: &NodeInternal<K, V> = transmute(*node);
                        target = &node.children[node.search(key)];
                    },
                    Leaf(ref leaf) => {
                        let leaf: &NodeLeaf<K, V> = transmute(*leaf);
                        target_leaf = Some(leaf);
                    },
                    Empty => {
                        return None;
                    },
                };
            }

            target_leaf.unwrap().find(key)
        }
    }
}

impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> Mutable for BTree<K, V> {
    fn clear(&mut self)
    {
        self.root = Empty;
    }
}

impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> MutableMap<K, V> for BTree<K, V> {
    fn swap(&mut self, key: K, value: V) -> Option<V>
    {
        match self.find_mut(&key) {
            Some(v) => {
                let mut value = value;
                util::swap(&mut value, v);
                return Some(value);
            },
            _ => ()
        }

        self.insert(key, value);
        None
    }

    fn pop(&mut self, key: &K) -> Option<V>
    {
        unsafe {
            match self.root.pop(key) {
                (_, found, false) => found,
                (_, found, true) => {
                    self.root.lift();
                    found
                }
            }
        }
    }

    #[inline(always)]
    fn find_mut<'a>(&'a mut self, key: &K) -> Option<&'a mut V>
    {
        unsafe {
            self.root.find_mut(key)
        }
    }

    fn insert(&mut self, key: K, value: V) -> bool
    {
        unsafe {
            match self.root.insert(&key, &value) {
                InsertDone(update) => update,
                // if the left key is only updated
                // on an insert, not an update so this
                // can return false
                InsertUpdateLeft(_) => false,
                Split => {
                    let (split_key, right) = match self.root {
                        Leaf(ref mut leaf) => {
                            let leaf: &mut NodeLeaf<K, V> = transmute(*leaf);
                            let (right, key) = leaf.split();
                            (key, Leaf(transmute(~right)))
                        },
                        Internal(ref mut node) => {
                            let node: &mut NodeInternal<K, V> = transmute(*node);
                            let (right, key) = node.split();
                            (key, Internal(transmute(~right)))

                        }
                        _ => fail!("this is impossible")
                    };
                    let mut left = Empty;

                    util::swap(&mut self.root, &mut left);

                    self.root = Internal(transmute(~NodeInternal::new(split_key, left, right)));
                    self.insert(key, value)
                }
            }
        }
    }
}

impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> Clone for BTree<K, V>
{
    fn clone(&self) -> BTree<K, V>
    {
        unsafe {
            self.root.inc_ref();
            BTree {
                root: self.root
            }
        }
    }
}

impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> BTree<K, V>
{
    pub fn new() -> BTree<K, V>
    {
        BTree {
            root: Empty
        }
    }

    pub fn iter<'a>(&'a self) -> BTreeIterator<'a, K, V>
    {
        unsafe {
            let (leaf, stack) = match self.root {
                Leaf(ref leaf) => {
                    let leaf: &NodeLeaf<K, V> = transmute(*leaf);
                    (Some(leaf.iter()), ~[])
                }
                Internal(ref node) => {
                    let node: &NodeInternal<K, V> = transmute(*node);
                    (None, ~[node.iter()])
                },
                Empty => (None, ~[])
            };
            BTreeIterator {
                leaf: leaf,
                stack: stack
            }
        }
    }
}

struct NodeIterator<'a, K, V>
{
    idx: uint,
    node: &'a NodeInternal<K, V>
}

impl<'a, K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> Iterator<NodeIteratorRes<'a,K,V>> for NodeIterator<'a, K, V>
{
    fn next(&mut self) -> Option<NodeIteratorRes<'a,K,V>>
    {
        unsafe {
            if self.idx < self.node.used {
                let idx = self.idx;
                self.idx += 1;
                match self.node.children[idx] {
                    Leaf(ref leaf) => {
                        let leaf: &NodeLeaf<K, V> = transmute(*leaf);
                        Some(LeafIter(leaf.iter()))
                    },
                    Internal(ref node) => {
                        let node: &NodeInternal<K, V> = transmute(*node);
                        Some(InternalIter(node.iter()))
                    }
                    Empty => None             
                }
            } else {
                None
            }
        }
    }
}

struct LeafIterator<'a, K, V>
{
    idx: uint,
    leaf: &'a NodeLeaf<K, V>
}

impl<'a, K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> Iterator<(&'a K, &'a V)> for LeafIterator<'a, K, V>
{
    fn next(&mut self) -> Option<(&'a K, &'a V)>
    {
        if self.idx < self.leaf.used {
            let idx = self.idx;
            self.idx += 1;
            Some((&self.leaf.keys[idx], &self.leaf.values[idx]))
        } else {
            None
        }
    }
}

struct BTreeIterator<'a, K, V>
{
    stack: ~[NodeIterator<'a, K, V>],
    leaf: Option<LeafIterator<'a, K, V>>
}

enum NodeIteratorRes<'a, K, V>
{
    InternalIter(NodeIterator<'a, K, V>),
    LeafIter(LeafIterator<'a, K, V>)
}

impl<'a, K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> Iterator<(&'a K, &'a V)> for BTreeIterator<'a, K, V>
{
    #[inline(always)]
    fn next(&mut self) -> Option<(&'a K, &'a V)>
    {
        loop {
            let res = match self.leaf {
                Some(ref mut    l) => l.next(),
                None => None
            };

            if res.is_some() {
                return res;
            } else {
                self.leaf = None;
            }

            if self.stack.len() == 0 {
                return None;
            } else {
                match self.stack[self.stack.len()-1].next() {
                    Some(InternalIter(node)) => self.stack.push(node),
                    Some(LeafIter(leaf)) => self.leaf = Some(leaf),
                    None => {
                        let _ = self.stack.pop();
                    }
                };
            }
        }
    }
}