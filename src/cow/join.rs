

pub struct JoinMapIterator<A, B> {a: A, b: B}

pub fn join_maps<KEY: TotalOrd, DATA_A, DATA_B, IterA: Iterator<(KEY, DATA_A)>, IterB: Iterator<(KEY, DATA_B)>>
    (a: IterA, b: IterB) -> JoinMapIterator<IterA, IterB> 
{
    JoinMapIterator {a: a, b: b}
}

impl<KEY: TotalOrd, DATA_A, DATA_B, IterA: Iterator<(KEY, DATA_A)>, IterB: Iterator<(KEY, DATA_B)>>
    Iterator<(KEY, (DATA_A, DATA_B))> for JoinMapIterator<IterA, IterB>
{
    #[inline(never)]
    fn next(&mut self) -> Option<(KEY, (DATA_A, DATA_B))>
    {
        let (mut key_a, mut data_a) = match self.a.next() {
            None => return None,
            Some((key, data)) => (key, data)
        };

        let (mut key_b, mut data_b) = match self.b.next() {
            None => return None,
            Some((key, data)) => (key, data)
        };

        loop {
            match key_a.cmp(&key_b) {
                Less => {
                    match self.a.next() {
                        None => return None,
                        Some((key, data)) => {
                            key_a = key;
                            data_a = data;
                        }
                    };
                },
                Equal => return Some((key_a, (data_a, data_b))),
                Greater => {
                    match self.b.next() {
                        None => return None,
                        Some((key, data)) => {
                            key_b = key;
                            data_b = data;
                        }
                    };
                }
            }
        }
    }
}

pub struct JoinSetIterator<A, B> {a: A, b: B}

pub fn join_sets<KEY: TotalOrd, IterA: Iterator<KEY>, IterB: Iterator<KEY>>
    (a: IterA, b: IterB) -> JoinSetIterator<IterA, IterB> 
{
    JoinSetIterator {a: a, b: b}
}

impl<KEY: TotalOrd, IterA: Iterator<KEY>, IterB: Iterator<KEY>>
    Iterator<KEY> for JoinSetIterator<IterA, IterB>
{
    #[inline(never)]
    fn next(&mut self) -> Option<KEY>
    {
        let mut key_a = match self.a.next() {
            None => return None,
            Some(key) => key
        };

        let mut key_b = match self.b.next() {
            None => return None,
            Some(key) => key
        };

        loop {
            match key_a.cmp(&key_b) {
                Less => {
                    match self.a.next() {
                        None => return None,
                        Some(key) => { key_a = key; }
                    };
                },
                Equal => return Some(key_a),
                Greater => {
                    match self.b.next() {
                        None => return None,
                        Some(key) => { key_b = key; }
                    };
                }
            }
        }
    }
}

pub struct JoinMapSetIterator<A, B> {set: A, map: B}

pub fn join_set_to_map<KEY: TotalOrd, DATA, SetIter: Iterator<KEY>, MapIter: Iterator<(KEY, DATA)>>
    (set: SetIter, map: MapIter) -> JoinMapSetIterator<SetIter, MapIter> 
{
    JoinMapSetIterator {set: set, map: map}
}

impl<KEY: TotalOrd, DATA, SetIter: Iterator<KEY>, MapIter: Iterator<(KEY, DATA)>>
    Iterator<(KEY, DATA)> for JoinMapSetIterator<SetIter, MapIter>
{
    #[inline(never)]
    fn next(&mut self) -> Option<(KEY, DATA)>
    {
        let mut key_set = match self.set.next() {
            None => return None,
            Some(key) => key
        };

        let (mut key_map, mut data) = match self.map.next() {
            None => return None,
            Some((key, data)) => (key, data)
        };

        loop {
            match key_set.cmp(&key_map) {
                Less => {
                    match self.set.next() {
                        None => return None,
                        Some(key) => { key_set = key; }
                    };
                },
                Equal => return Some((key_set, data)),
                Greater => {
                    match self.map.next() {
                        None => return None,
                        Some((key, d)) => { 
                            key_map = key;
                            data = d;
                        }
                    };
                }
            }
        }
    }
}