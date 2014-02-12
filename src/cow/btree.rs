use CowArc;

use std::mem;

use std::default::Default;
use std::iter::range_step;

static LEAF_SIZE: uint = 31;
static INTERNAL_SIZE: uint = 42;

struct NodeLeaf<K, V> {
    used:   uint,
    keys:   [K, ..LEAF_SIZE],
    values: [V, ..LEAF_SIZE] 
}

struct NodeInternal<K, V> {
    used:      uint,
    total_len: uint,
    keys:      [K, ..INTERNAL_SIZE-1],
    children:  [Node<K, V>, ..INTERNAL_SIZE]
}

enum Node<K, V> {
    Empty,
    Internal(CowArc<NodeInternal<K, V>>),
    Leaf(CowArc<NodeLeaf<K, V>>),
}

pub struct BTreeMap<K, V> {
    priv root: Node<K, V>
}

enum InsertAction<K, V> {
    InsertDone(bool),
    Split(K, V),
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
            match *self {
                Empty => Empty,
                Internal(ref node) => Internal(node.clone()),
                Leaf(ref leaf) => Leaf(leaf.clone())
            }
    }
}

impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> Node<K, V>
{
    #[inline(always)]
    fn insert(&mut self, key: K, value: V) -> InsertAction<K, V>
    {
        match *self {
            Empty => {
                *self = Leaf(CowArc::new(NodeLeaf::new()));
                self.insert(key, value)
            },
            Leaf(ref mut leaf) => {
                leaf.get_mut().insert(key, value)
            },
            Internal(ref mut node) => {
                node.get_mut().insert(key, value)
            }
        }
    }

    fn pop(&mut self, key: &K) -> (Option<K>, Option<V>, bool)
    {
        match *self {
            Empty => (None, None, false),
            Leaf(ref mut leaf) => leaf.get_mut().pop(key),
            Internal(ref mut node) => node.get_mut().pop(key)
        }        
    }

    fn split(&mut self) -> (Node<K, V>, K)
    {
        match *self {
            Leaf(ref mut leaf) => {
                let (leaf, key) = leaf.get_mut().split();
                (Leaf(CowArc::new(leaf)), key)
            },
            Internal(ref mut node) => {
                let (node, key) = node.get_mut().split();
                (Internal(CowArc::new(node)), key)
            },
            _ => {
                fail!("unsupported split");
            }
        }
    }

    fn len(&self) -> uint
    {
        match *self {
            Empty => {0u},
            Leaf(ref leaf) => leaf.get().len(),
            Internal(ref node) => node.get().len()
        }     
    }

    fn find_mut<'a>(&'a mut self, key: &K) -> Option<&'a mut V>
    {
        match *self {
            Empty => None,
            Leaf(ref mut leaf) => leaf.get_mut().find_mut(key),
            Internal(ref mut node) => node.get_mut().find_mut(key)
        }
    }

    // move the lowest key from other to self iff node is has extra keys
    fn rotate_right(&mut self, src: &mut Node<K, V>) -> bool
    {
        match (self, src) {
            (&Leaf(ref mut sink), &Leaf(ref mut src)) => {
                sink.get_mut().rotate_right(src.get_mut())
            },
            (&Internal(ref mut sink), &Internal(ref mut src)) => {
                sink.get_mut().rotate_right(src.get_mut())
            },
            (_, _) => {
                fail!("both nodes should be of the same type");
            }
        }
    }

    // move highest key from src to self iff node is has extra keys
    fn rotate_left(&mut self, src: &mut Node<K, V>) -> bool
    {
        match (self, src) {
            (&Leaf(ref mut sink), &Leaf(ref mut src)) => {
                sink.get_mut().rotate_left(src.get_mut())
            },
            (&Internal(ref mut sink), &Internal(ref mut src)) => {
                sink.get_mut().rotate_left(src.get_mut())
            },
            (_, _) => {
                fail!("both nodes should be of the same type");
            }
        }
    }

    // move highest key from src to self iff node is has extra keys
    fn merge(&mut self, src: Node<K, V>)
    {
        match (self, src) {
            (&Leaf(ref mut sink), Leaf(ref mut src)) => {
                sink.get_mut().merge(src.get_mut());
            },
            (&Internal(ref mut sink), Internal(ref mut src)) => {
                sink.get_mut().merge(src.get_mut());
            },
            (_, _) => {
                fail!("both nodes should be of the same type");
            }
        }
    }

    fn max_key(&self) -> K
    {
        match *self {
            Leaf(ref leaf) => leaf.get().max_key(),
            Internal(ref node) => node.get().max_key(),
            Empty => fail!("invalid node")
        }
    }

    fn lift(&mut self)
    {
        let depleted = match *self {
            Internal(ref node) => node.get().used == 1,
            Leaf(ref leaf) => leaf.get().used == 0,
            _ => false
        };

        if depleted {
            let mut child = Empty;
            match *self {
                Internal(ref mut node) => {
                    mem::swap(&mut child, &mut node.get_mut().children[0]);
                },
                Leaf(_) => {},
                _ => fail!("invalid node")
            }
            *self = child;
        }
    }
}

impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> NodeInternal<K, V>
{
    fn new(key: K, left: Node<K, V>, right: Node<K, V>) -> NodeInternal<K, V>
    {
        let mut node = NodeInternal::new_empty();
        node.used = 2;
        node.keys[0] = key;
        node.total_len = right.len() + left.len();
        node.children[0] = left;
        node.children[1] = right;
        node
    }

    fn new_empty() -> NodeInternal<K, V>
    {
        NodeInternal {
            used: 0,
            total_len: 0,
            keys: [default(), default(), default(), default(),  // 0-3
                   default(), default(), default(), default(),  // 4-7
                   default(), default(), default(), default(),  // 8-11
                   default(), default(), default(), default(),  // 12-15
                   default(), default(), default(), default(),  // 16-19
                   default(), default(), default(), default(),  // 20-23
                   default(), default(), default(), default(),  // 24-27
                   default(), default(), default(), default(),  // 28-31
                   default(), default(), default(), default(),  // 32-35
                   default(), default(), default(), default(),  // 36-39
                   default()],                                  // 40
            children: [Empty, Empty, Empty, Empty,  // 0-3
                       Empty, Empty, Empty, Empty,  // 4-7
                       Empty, Empty, Empty, Empty,  // 8-11
                       Empty, Empty, Empty, Empty,  // 12-15
                       Empty, Empty, Empty, Empty,  // 16-19
                       Empty, Empty, Empty, Empty,  // 20-23
                       Empty, Empty, Empty, Empty,  // 24-27
                       Empty, Empty, Empty, Empty,  // 28-31
                       Empty, Empty, Empty, Empty,  // 32-35
                       Empty, Empty, Empty, Empty,  // 36-39
                       Empty, Empty],               // 40-41
        }
    }

    #[inline(always)]
    fn insert(&mut self, key: K, value: V) -> InsertAction<K, V>
    {
        let idx = self.search(&key);

        match self.children[idx].insert(key, value) {
            InsertDone(updated) => {
                if !updated {
                    self.total_len += 1;
                }
                InsertDone(updated)
            },
            Split(key, value) => {
                if self.used == INTERNAL_SIZE {
                    Split(key, value)
                } else {
                    let (right, split_key) = self.children[idx].split();
                    let mut right = right;
                    let mut split_key = split_key;

                    let idx = idx + 1;

                    // move items left
                    for j in range(idx-1, self.used) {
                        mem::swap(&mut self.keys[j], &mut split_key);
                    }

                    for j in range(idx, self.used+1) {
                        mem::swap(&mut self.children[j], &mut right);
                    }

                    self.used += 1;
                    self.insert(key, value)
                }
            },
            InsertUpdateLeft(left) => {
                self.total_len += 1;
                if idx != self.used {
                    self.keys[idx] = left;
                    InsertUpdateLeft(self.keys[self.used-2].clone())
                } else {
                    InsertDone(false)
                }
            } 
        }
    }

    #[inline(always)]
    fn redist(&mut self, idx: uint)
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
        mem::swap(&mut child, &mut self.children[insert+1]);

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
    }

    fn pop(&mut self, key: &K) -> (Option<K>, Option<V>, bool)
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

        if value.is_some() {
            self.total_len -= 1;
        }

        (key, value, self.used < INTERNAL_SIZE / 2)
    }

    fn find_mut<'a>(&'a mut self, key: &K) -> Option<&'a mut V>
    {
        self.children[self.search(key)].find_mut(key)
    }

    #[inline(always)]
    fn search(&self, key: &K) -> uint
    {
        let mut end = self.used-1;
        let mut start = 0u;
        for i in range_step(2u, end, 8u) {
            match key.cmp(&self.keys[i]) {
                Less | Equal => {
                    end = i;
                    break;
                },
                Greater => {
                    start = i;
                }
            }
        }

        while end > start {
            match key.cmp(&self.keys[start]) {
                Equal | Less => return start,
                Greater => start += 1,
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

// this is about 15% slower then search
//    #[inline(always)]
//    fn bsearch(&self, key: &K) -> uint
//    {
//        let mut start = 0u;
//        let mut end = self.used-1;
//
//        while end > start {
//            let mid = start + ((end-start) / 2);
//
//            match key.cmp(&self.keys[mid]) {
//                Less => end = mid,
//                Equal => return mid,
//                Greater => start = mid+1,
//            }
//        }
//        if start != self.used-1 {
//            match key.cmp(&self.keys[start]) {
//                Less | Equal => start,
//                Greater => start+1,
//            }
//        } else {
//            start
//        }
//    }

    #[inline(always)]
    fn split(&mut self) -> (NodeInternal<K, V>, K)
    {
        let mut right = NodeInternal::new_empty();

        for (dst, src) in range(INTERNAL_SIZE / 2, self.used).enumerate() {
            mem::swap(&mut right.children[dst], &mut self.children[src]);
        }

        for (dst, src) in range(INTERNAL_SIZE / 2, self.used-1).enumerate() {
            mem::swap(&mut right.keys[dst], &mut self.keys[src]);
        }

        right.used = self.used - INTERNAL_SIZE / 2;
        self.used =  INTERNAL_SIZE / 2;

        let mut key: K = default();
        mem::swap(&mut key, &mut self.keys[self.used-1]);

        (right, key)
    }

    fn len(&self) -> uint
    {
        self.total_len
    }

    #[inline(always)]
    fn rotate_left(&mut self, left: &mut NodeInternal<K, V>) -> bool
    {
        if left.used > INTERNAL_SIZE / 2 {
            self.keys[self.used-1] = self.children[self.used-1].max_key();
            for i in range(0, left.used-2) {
                left.keys.swap(i, i+1);
            }

            mem::swap(&mut self.children[self.used], &mut left.children[0]);
            for i in range(0, left.used-1) {
                left.children.swap(i, i+1);
            }

            let size = self.children[self.used].len();
            left.used -= 1;
            left.total_len -= size;
            self.used += 1;
            self.total_len += size;
            true
        } else {
            false
        }
    }

    #[inline(always)]
    fn rotate_right(&mut self, right: &mut NodeInternal<K, V>) -> bool
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
            mem::swap(&mut self.children[0], &mut right.children[right.used-1]);

            let size = self.children[0].len();
            right.used -= 1;
            right.total_len -= size;
            self.used += 1;
            self.total_len += size;
            true
        } else {
            false
        }
    }

    #[inline(always)]
    fn merge(&mut self, right: &mut NodeInternal<K, V>)
    {
        self.keys[self.used-1] = self.children[self.used-1].max_key();
        for (src, dst) in range(self.used, self.used+right.used).enumerate() {
            mem::swap(&mut right.keys[src], &mut self.keys[dst]);
            mem::swap(&mut right.children[src], &mut self.children[dst]);
        }
        self.total_len += right.total_len;
        self.used += right.used;
    }

    fn max_key(&self) -> K
    {
        self.children[self.used-1].max_key()
    }

    fn iter<'a>(&'a self) -> NodeIterator<'a, K, V>
    {
        NodeIterator {
            idx: 0,
            node: self
        }
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
        new.total_len = self.total_len;

        new
    }
}


impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> NodeLeaf<K, V>
{
    fn new() -> NodeLeaf<K, V>
    {
        NodeLeaf {
            used: 0,
            keys: [default(), default(), default(), default(),  // 0-3
                   default(), default(), default(), default(),  // 4-7
                   default(), default(), default(), default(),  // 8-11
                   default(), default(), default(), default(),  // 12-15
                   default(), default(), default(), default(),  // 16-19
                   default(), default(), default(), default(),  // 20-23
                   default(), default(), default(), default(),  // 24-27
                   default(), default(), default()],            // 28-30
            values: [default(), default(), default(), default(),  // 0-3
                     default(), default(), default(), default(),  // 4-7
                     default(), default(), default(), default(),  // 8-11
                     default(), default(), default(), default(),  // 12-15
                     default(), default(), default(), default(),  // 16-19
                     default(), default(), default(), default(),  // 20-23
                     default(), default(), default(), default(),  // 24-27
                     default(), default(), default()]             // 28-30
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

    #[inline(always)]
    fn insert(&mut self, key: K, value: V) -> InsertAction<K, V>
    {
        if self.used == LEAF_SIZE {
            Split(key, value)
        } else {
            let (found, insert) = self.search_key(&key);

            // update
            if insert != self.used && found {
                self.values[insert] = value;
                InsertDone(true)
            // insert
            } else {
                let mut key = key;
                let mut value = value;

                self.used += 1;

                for j in range(insert, self.used) {
                    mem::swap(&mut self.keys[j], &mut key);
                    mem::swap(&mut self.values[j], &mut value);
                }

                if insert == self.used {
                    InsertUpdateLeft(self.keys[self.used-1].clone())
                } else {
                    InsertDone(false)
                }
            }
        }
    }

    #[inline(always)]
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
            mem::swap(&mut self.keys[i], &mut key);
            mem::swap(&mut self.values[i], &mut value);
        }

        self.used -= 1;

        (if self.used != 0 {
             Some(self.keys[self.used-1].clone())
         } else {
             None
         },
         Some(value),
         self.used < LEAF_SIZE / 2)
    }

    #[inline(always)]
    fn find<'a>(&'a self, key: &K) -> Option<&'a V>
    {
        match self.search(key) {
            Some(idx) => Some(&self.values[idx]),
            None => None
        }
    }

    #[inline(always)]
    fn find_mut<'a>(&'a mut self, key: &K) -> Option<&'a mut V>
    {
        match self.search(key) {
            Some(idx) => Some(&mut self.values[idx]),
            None => None
        }
    }

    #[inline(always)]
    fn split(&mut self) -> (NodeLeaf<K, V>, K)
    {
        let mut right = NodeLeaf::new();

        for (dst, src) in range(LEAF_SIZE / 2, self.used).enumerate() {
            mem::swap(&mut right.keys[dst], &mut self.keys[src]);
            mem::swap(&mut right.values[dst], &mut self.values[src]);
        }

        right.used = self.used - LEAF_SIZE / 2;
        self.used =  LEAF_SIZE / 2;

        (right, self.keys[self.used-1].clone())
    }

    #[inline(always)]
    fn len(&self) -> uint
    {
        self.used
    }

    #[inline(always)]
    fn rotate_left(&mut self, left: &mut NodeLeaf<K, V>) -> bool
    {
        if left.used > LEAF_SIZE / 2 {
            mem::swap(&mut self.keys[self.used], &mut left.keys[0]);
            for i in range(0, left.used-1) {
                left.keys.swap(i, i+1);
            }

            mem::swap(&mut self.values[self.used], &mut left.values[0]);
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

    #[inline(always)]
    fn rotate_right(&mut self, right: &mut NodeLeaf<K, V>) -> bool
    {
        if right.used > LEAF_SIZE / 2 {
            for i in range(0, self.used+1) {
                let i = self.used + 1 - i;
                self.keys.swap(i, i-1);
            }
            mem::swap(&mut self.keys[0], &mut right.keys[right.used-1]);

            for i in range(0, self.used+1) {
                let i = self.used + 1 - i;
                self.values.swap(i, i-1);
            }
            mem::swap(&mut self.values[0], &mut right.values[right.used-1]);

            right.used -= 1;
            self.used += 1;
            true
        } else {
            false
        }
    }

    #[inline(always)]
    fn merge(&mut self, right: &mut NodeLeaf<K, V>)
    {
        for (src, dst) in range(self.used, self.used+right.used).enumerate() {
            mem::swap(&mut right.keys[src], &mut self.keys[dst]);
            mem::swap(&mut right.values[src], &mut self.values[dst]);
        }
        self.used += right.used;
    }

    #[inline(always)]
    fn max_key(&self) -> K
    {
        self.keys[self.used-1].clone()
    }

    #[inline(always)]
    fn iter<'a>(&'a self) -> LeafIterator<'a, K, V>
    {
        LeafIterator {
            idx: 0,
            leaf: self
        }
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

impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> Container for BTreeMap<K, V> {
    fn len(&self) -> uint
    {
        self.root.len()
    }
}

impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> Map<K, V> for BTreeMap<K, V> {
    #[inline(always)]
    fn find<'a>(&'a self, key: &K) -> Option<&'a V>
    {
        let mut target = &self.root;
        let mut target_leaf: Option<&NodeLeaf<K, V>> = None;

        while target_leaf.is_none() {
            match *target {
                Internal(ref node) => {
                    target = &node.get().children[node.get().search(key)];
                },
                Leaf(ref leaf) => {
                    target_leaf = Some(leaf.get());
                },
                Empty => {
                    return None;
                },
            };
        }

        target_leaf.unwrap().find(key)
    }
}

impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> Mutable for BTreeMap<K, V> {
    fn clear(&mut self)
    {
        self.root = Empty;
    }
}

impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> MutableMap<K, V> for BTreeMap<K, V> {
    #[inline(always)]
    fn swap(&mut self, key: K, value: V) -> Option<V>
    {
        match self.find_mut(&key) {
            Some(v) => {
                let mut value = value;
                mem::swap(&mut value, v);
                return Some(value);
            },
            _ => ()
        }

        self.insert(key, value);
        None
    }

    #[inline(always)]
    fn pop(&mut self, key: &K) -> Option<V>
    {
        match self.root.pop(key) {
            (_, found, false) => found,
            (_, found, true) => {
                self.root.lift();
                found
            }
        }
    }

    #[inline(always)]
    fn find_mut<'a>(&'a mut self, key: &K) -> Option<&'a mut V>
    {
        self.root.find_mut(key)
    }

    #[inline(always)]
    fn insert(&mut self, key: K, value: V) -> bool
    {
        match self.root.insert(key, value) {
            InsertDone(update) => update,
            // if the left key is only updated
            // on an insert, not an update so this
            // can return false
            InsertUpdateLeft(_) => false,
            Split(key, value) => {
                let (split_key, right) = match self.root {
                    Leaf(ref mut leaf) => {
                        let (right, key) = leaf.get_mut().split();
                        (key, Leaf(CowArc::new(right)))
                    },
                    Internal(ref mut node) => {
                        let (right, key) = node.get_mut().split();
                        (key, Internal(CowArc::new(right)))

                    }
                    _ => fail!("this is impossible")
                };
                let mut left = Empty;

                mem::swap(&mut self.root, &mut left);

                self.root = Internal(CowArc::new(NodeInternal::new(split_key, left, right)));
                self.insert(key, value)
            }
        }
    }
}

impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> Clone for BTreeMap<K, V>
{
    fn clone(&self) -> BTreeMap<K, V>
    {
        BTreeMap {
            root: self.root.clone()
        }
    }
}

impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> BTreeMap<K, V>
{
    pub fn new() -> BTreeMap<K, V>
    {
        //println!("{:?} {:?} {:?}",
        //        std::mem::size_of::<Node<K, V>>(),
        //        std::mem::size_of::<NodeLeaf<K, V>>(),
        //        std::mem::size_of::<NodeInternal<K, V>>()
        //);
        BTreeMap {
            root: Empty
        }
    }

    pub fn iter<'a>(&'a self) -> BTreeMapIterator<'a, K, V>
    {
        let (leaf, stack) = match self.root {
            Leaf(ref leaf) => {
                (Some(leaf.get().iter()), ~[])
            }
            Internal(ref node) => {
                (None, ~[node.get().iter()])
            },
            Empty => (None, ~[])
        };
        BTreeMapIterator {
            leaf: leaf,
            stack: stack,
            current: 0,
            end: self.len()
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
        if self.idx < self.node.used {
            let idx = self.idx;
            self.idx += 1;
            match self.node.children[idx] {
                Leaf(ref leaf) => {
                    Some(LeafIter(leaf.get().iter()))
                },
                Internal(ref node) => {
                    Some(InternalIter(node.get().iter()))
                }
                Empty => None             
            }
        } else {
            None
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

pub struct BTreeMapIterator<'a, K, V>
{
    stack: ~[NodeIterator<'a, K, V>],
    leaf: Option<LeafIterator<'a, K, V>>,
    current: uint,
    end: uint
}

enum NodeIteratorRes<'a, K, V>
{
    InternalIter(NodeIterator<'a, K, V>),
    LeafIter(LeafIterator<'a, K, V>)
}

impl<'a, K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> Iterator<(&'a K, &'a V)> for BTreeMapIterator<'a, K, V>
{
    #[inline(always)]
    fn next(&mut self) -> Option<(&'a K, &'a V)>
    {
        if self.current == self.end {
            return None;
        }
        loop {
            let res = match self.leaf {
                Some(ref mut    l) => l.next(),
                None => None
            };

            if res.is_some() {
                self.current += 1;
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

impl<K: Default+Clone+TotalOrd+Send+Freeze, V: Default+Clone+Send+Freeze> Default for BTreeMap<K, V>
{
    fn default() -> BTreeMap<K, V>
    {
        BTreeMap::new()
    }
}

pub struct BTreeSet<T> {
    priv map: BTreeMap<T, ()>
}

pub struct BTreeSetIterator<'a, T>
{
    priv mapiter: BTreeMapIterator<'a, T, ()>
}

impl<T: Default+Clone+TotalOrd+Send+Freeze> BTreeSet<T>
{
    pub fn new() -> BTreeSet<T>
    {
        BTreeSet{map: BTreeMap::new()}
    }

    pub fn iter<'a>(&'a self) -> BTreeSetIterator<'a, T>
    {
        BTreeSetIterator {
            mapiter: self.map.iter()
        }
    }
}

impl<T: Default+Clone+TotalOrd+Send+Freeze> Container for BTreeSet<T> {
    fn len(&self) -> uint { self.map.len() }
}

impl<T: Default+Clone+TotalOrd+Send+Freeze> Mutable for BTreeSet<T> {
    fn clear(&mut self) { self.map.clear() }
}

impl<T: Default+Clone+TotalOrd+Send+Freeze> Set<T> for BTreeSet<T> {
    fn contains(&self, value: &T) -> bool { self.map.find(value).is_some() }

    fn is_disjoint(&self, other: &BTreeSet<T>) -> bool {
        self.iter().all(|v| !other.contains(v))
    }

    fn is_subset(&self, other: &BTreeSet<T>) -> bool {
        self.iter().all(|v| other.contains(v))
    }

    fn is_superset(&self, other: &BTreeSet<T>) -> bool {
        other.is_subset(self)
    }
}

impl<T: Default+Clone+TotalOrd+Send+Freeze> MutableSet<T> for BTreeSet<T>
{
    fn insert(&mut self, value: T) -> bool
    {
        self.map.insert(value, ())
    }

    fn remove(&mut self, value: &T) -> bool
    {
        self.map.remove(value)
    }
}

impl<'a, K: Default+Clone+TotalOrd+Send+Freeze> Iterator<&'a K> for BTreeSetIterator<'a, K>
{
    #[inline(always)]
    fn next(&mut self) -> Option<(&'a K)>
    {
        match self.mapiter.next() {
            Some((k, _)) => Some(k),
            None => None
        }
    }
}

impl<T: Default+Clone+TotalOrd+Send+Freeze> Clone for BTreeSet<T>
{
    fn clone(&self) -> BTreeSet<T>
    {
        BTreeSet {
            map: self.map.clone()
        }
    }
}

impl<T: Default+Clone+TotalOrd+Send+Freeze> Default for BTreeSet<T>
{
    fn default() -> BTreeSet<T>
    {
        BTreeSet::new()
    }
}