pub use hashmap::*;

mod hashmap {
    use std::collections::HashMap;
    use std::hash::Hash;
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
            let old_value = self.0.remove(&key);
            self.0.insert(key.clone(), value);
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
