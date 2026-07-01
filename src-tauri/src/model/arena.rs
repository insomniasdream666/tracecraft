use serde::{
    de::{Deserialize, Deserializer, MapAccess, Visitor},
    ser::{Serialize, SerializeMap, Serializer},
};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Arena<T> {
    items: Vec<Option<T>>,
    free_list: Vec<usize>,
}

impl<T: Serialize> Serialize for Arena<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (index, item) in self.iter() {
            map.serialize_entry(&index, item)?;
        }
        map.end()
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Arena<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ArenaVisitor<T>(std::marker::PhantomData<T>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for ArenaVisitor<T> {
            type Value = Arena<T>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a map of integer indices to values")
            }

            fn visit_map<M: MapAccess<'de>>(self, mut access: M) -> Result<Self::Value, M::Error> {
                let mut arena = Arena::new();
                let mut max_index = 0usize;
                let mut entries: Vec<(usize, T)> = Vec::new();

                while let Some((index, value)) = access.next_entry::<usize, T>()? {
                    if index >= max_index {
                        max_index = index + 1;
                    }
                    entries.push((index, value));
                }

                arena.items.resize_with(max_index, || None);
                arena.free_list.clear();

                for (index, value) in entries {
                    arena.items[index] = Some(value);
                }

                Ok(arena)
            }
        }

        deserializer.deserialize_map(ArenaVisitor(std::marker::PhantomData))
    }
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            free_list: Vec::new(),
        }
    }

    pub fn insert(&mut self, item: T) -> usize {
        if let Some(index) = self.free_list.pop() {
            self.items[index] = Some(item);
            index
        } else {
            let index = self.items.len();
            self.items.push(Some(item));
            index
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index).and_then(|i| i.as_ref())
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.items.get_mut(index).and_then(|i| i.as_mut())
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        let item = self.items.get_mut(index)?.take();
        if item.is_some() {
            self.free_list.push(index);
        }
        item
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| item.as_ref().map(|t| (i, t)))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        self.items
            .iter_mut()
            .enumerate()
            .filter_map(|(i, item)| item.as_mut().map(|t| (i, t)))
    }

    pub fn len(&self) -> usize {
        self.items.len() - self.free_list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.free_list.clear();
    }

    pub fn insert_at(&mut self, index: usize, item: T) {
        if index < self.items.len() {
            self.items[index] = Some(item);
        } else {
            while self.items.len() < index {
                self.items.push(None);
                self.free_list.push(self.items.len() - 1);
            }
            self.items.push(Some(item));
        }
        self.free_list.retain(|&i| i != index);
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_arena_is_empty() {
        let arena: Arena<i32> = Arena::new();
        assert!(arena.is_empty());
        assert_eq!(arena.len(), 0);
    }

    #[test]
    fn test_insert_and_get() {
        let mut arena = Arena::new();
        let idx = arena.insert(42);
        assert_eq!(idx, 0);
        assert_eq!(*arena.get(idx).unwrap(), 42);
    }

    #[test]
    fn test_insert_multiple() {
        let mut arena = Arena::new();
        let a = arena.insert(10);
        let b = arena.insert(20);
        let c = arena.insert(30);
        assert_eq!(a, 0);
        assert_eq!(b, 1);
        assert_eq!(c, 2);
        assert_eq!(*arena.get(1).unwrap(), 20);
    }

    #[test]
    fn test_remove_and_reuse() {
        let mut arena = Arena::new();
        arena.insert(10);
        arena.insert(20);
        arena.insert(30);
        assert_eq!(arena.len(), 3);
        arena.remove(1);
        assert_eq!(arena.len(), 2);
        let new_idx = arena.insert(40);
        assert_eq!(new_idx, 1);
        assert_eq!(*arena.get(1).unwrap(), 40);
    }

    #[test]
    fn test_iter() {
        let mut arena = Arena::new();
        arena.insert("a");
        arena.insert("b");
        arena.insert("c");
        let collected: Vec<&str> = arena.iter().map(|(_, v)| *v).collect();
        assert_eq!(collected, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut arena: Arena<i32> = Arena::new();
        assert!(arena.remove(0).is_none());
    }

    #[test]
    fn test_clear() {
        let mut arena = Arena::new();
        arena.insert(1);
        arena.insert(2);
        arena.clear();
        assert!(arena.is_empty());
    }
}
