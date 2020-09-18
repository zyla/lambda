pub use linear::*;

#[allow(dead_code)]
mod hashmap {
    use std::collections::{hash_map::Entry, HashMap};
    use std::hash::Hash;
    use std::mem;
    use std::ops::{Deref, DerefMut};

    #[derive(Debug, Clone)]
    pub struct Context<K, V>(HashMap<K, V>);

    impl<K, V> Default for Context<K, V> {
        fn default() -> Self {
            Context(Default::default())
        }
    }

    impl<K: Hash + Clone + Eq, V> Context<K, V> {
        pub fn with(&mut self, key: K, value: V) -> Guard<K, V> {
            let old_value = match self.0.entry(key.clone()) {
                Entry::Occupied(mut e) => Some(mem::replace(e.get_mut(), value)),
                Entry::Vacant(e) => {
                    e.insert(value);
                    None
                }
            };
            Guard {
                ctx: self,
                key: Some(key),
                old_value,
            }
        }

        pub fn get(&self, key: &K) -> Option<&V> {
            self.0.get(key)
        }

        pub fn insert(&mut self, key: K, value: V) {
            self.0.insert(key, value);
        }
    }

    pub struct Guard<'a, K: Hash + Eq, V> {
        ctx: &'a mut Context<K, V>,
        key: Option<K>,
        old_value: Option<V>,
    }

    impl<'a, K: Hash + Eq, V> Deref for Guard<'a, K, V> {
        type Target = Context<K, V>;
        fn deref(&self) -> &Self::Target {
            self.ctx
        }
    }

    impl<'a, K: Hash + Eq, V> DerefMut for Guard<'a, K, V> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.ctx
        }
    }

    impl<'a, K: Hash + Eq, V> Drop for Guard<'a, K, V> {
        fn drop(&mut self) {
            let key = self.key.take().unwrap();
            match self.old_value.take() {
                Some(value) => self.0.insert(key, value),
                None => self.0.remove(&key),
            };
        }
    }
}

mod linear {
    use std::mem;
    use std::ops::{Deref, DerefMut};

    #[derive(Debug, Clone)]
    pub struct Context<K, V>(Vec<Option<(K, V)>>);

    impl<K, V> Default for Context<K, V> {
        fn default() -> Self {
            Context(Default::default())
        }
    }

    impl<K: Clone + Eq, V> Context<K, V> {
        pub fn with(&mut self, key: K, value: V) -> Guard<K, V> {
            let old_entry = self.insert(key, value);
            Guard {
                ctx: self,
                entry: Some(old_entry),
            }
        }

        pub fn get(&self, key: &K) -> Option<&V> {
            self.0
                .iter()
                .find(|entry| match entry {
                    Some((k, _)) => k == key,
                    None => false,
                })
                .map(|e| &e.as_ref().unwrap().1)
        }

        pub fn insert(&mut self, key: K, value: V) -> (usize, Option<(K, V)>) {
            match self.0.iter().position(matches_key_or_empty(&key)) {
                Some(index) => (index, mem::replace(&mut self.0[index], Some((key, value)))),
                None => {
                    let index = self.0.len();
                    self.0.push(Some((key, value)));
                    (index, None)
                }
            }
        }
    }

    fn matches_key_or_empty<K: Eq, V>(key: &K) -> impl FnMut(&Option<(K, V)>) -> bool + '_ {
        move |entry| match entry {
            Some((k, _)) => k == key,
            None => true,
        }
    }

    pub struct Guard<'a, K, V> {
        ctx: &'a mut Context<K, V>,
        entry: Option<(usize, Option<(K, V)>)>,
    }

    impl<'a, K, V> Deref for Guard<'a, K, V> {
        type Target = Context<K, V>;
        fn deref(&self) -> &Self::Target {
            self.ctx
        }
    }

    impl<'a, K, V> DerefMut for Guard<'a, K, V> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.ctx
        }
    }

    impl<'a, K, V> Drop for Guard<'a, K, V> {
        fn drop(&mut self) {
            if let Some((index, entry)) = self.entry.take() {
                self.ctx.0[index] = entry;
            }
        }
    }
}
