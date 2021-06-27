#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ptr::replace;

#[cfg(test)]
mod tests;
#[cfg(feature = "serde")]
mod serde;

struct KeyPtr<K> {
    k: *const K,
}

#[derive(Hash, PartialEq, Eq)]
#[repr(transparent)]
struct Qey<Q: ?Sized>(Q);

impl<Q: ?Sized> Qey<Q> {
    fn from_ref(q: &Q) -> &Self {
        unsafe { std::mem::transmute(q) }
    }
}

impl<K, Q: ?Sized> Borrow<Qey<Q>> for KeyPtr<K>
where
    K: Borrow<Q>,
{
    fn borrow(&self) -> &Qey<Q> {
        Qey::from_ref(unsafe { (*self.k).borrow() })
    }
}

impl<K: Hash> Hash for KeyPtr<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe { (*self.k).hash(state) }
    }
}

impl<K: PartialEq> PartialEq for KeyPtr<K> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { (*self.k).eq(&*other.k) }
    }
}

impl<K: Eq> Eq for KeyPtr<K> {}

pub struct LinkedHashMap<K, V, S = std::collections::hash_map::RandomState> {
    hash_map: HashMap<KeyPtr<K>, *mut Node<K, V>, S>,
    head: Option<*mut Node<K, V>>,
    tail: Option<*mut Node<K, V>>,
    marker: PhantomData<Node<K, V>>,
}

pub struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<*mut Node<K, V>>,
    next: Option<*mut Node<K, V>>,
}

impl<K, V> Node<K, V> {
    pub fn into_ptr(_self: Self) -> *mut Self {
        Box::into_raw(Box::new(_self))
    }
}

impl<K, V, S> LinkedHashMap<K, V, S> {
    pub fn with_hasher(hasher: S) -> LinkedHashMap<K, V, S> {
        LinkedHashMap {
            hash_map: HashMap::with_hasher(hasher),
            head: None,
            tail: None,
            marker: Default::default(),
        }
    }

    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        LinkedHashMap {
            hash_map: HashMap::with_capacity_and_hasher(capacity, hasher),
            head: None,
            tail: None,
            marker: Default::default(),
        }
    }
}

impl<K, V> LinkedHashMap<K, V, std::collections::hash_map::RandomState>
where
    K: Hash + Eq,
{
    pub fn new() -> Self {
        LinkedHashMap {
            hash_map: HashMap::new(),
            ..Default::default()
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        LinkedHashMap {
            hash_map: HashMap::with_capacity(capacity),
            ..Default::default()
        }
    }

    #[inline]
    pub fn push_front(&mut self, key: K, value: V) -> Option<(&K, &V)> {
        unsafe {
            if let Some(node) = self.hash_map.get(&KeyPtr { k: &key }) {
                replace(&mut (**node).value, value);
                Some((&(**node).key, &(**node).value))
            } else {
                let node = Node::into_ptr(Node {
                    key,
                    value,
                    prev: None,
                    next: self.head,
                });

                self.hash_map.insert(KeyPtr { k: &(*node).key }, node);

                let node = Some(node);
                match self.head {
                    None => self.tail = node,
                    Some(head) => (*head).prev = node,
                }

                self.head = node;
                None
            }
        }
    }

    #[inline]
    pub fn pop_front_node(&mut self) -> Option<Box<Node<K, V>>> {
        self.head
            .map(|node| unsafe {
                self.head = (*node).next;

                match self.head {
                    None => self.tail = None,
                    Some(head) => (*head).prev = None,
                }

                self.hash_map
                    .remove(&KeyPtr { k: &(*node).key })
                    .map(|node| Box::from_raw(node))
            })
            .flatten()
    }

    #[inline]
    pub fn pop_front(&mut self) -> Option<(K, V)> {
        self.pop_front_node().map(|node| (node.key, node.value))
    }

    #[inline]
    pub fn front(&self) -> Option<(&K, &V)> {
        self.head
            .map(|node| unsafe { (&(*node).key, &(*node).value) })
    }

    #[inline]
    pub fn push_back(&mut self, key: K, value: V) -> Option<(&K, &V)> {
        unsafe {
            if let Some(node) = self.hash_map.get(&KeyPtr { k: &key }) {
                replace(&mut (**node).value, value);
                Some((&(**node).key, &(**node).value))
            } else {
                let node = Node::into_ptr(Node {
                    key,
                    value,
                    prev: self.tail,
                    next: None,
                });
                self.hash_map.insert(KeyPtr { k: &(*node).key }, node);
                let node = Some(node);
                if let Some(tail) = self.tail {
                    (*tail).next = node
                } else {
                    self.head = node;
                }
                self.tail = node;
                None
            }
        }
    }

    #[inline]
    pub fn pop_back_node(&mut self) -> Option<Box<Node<K, V>>> {
        self.tail
            .map(|node| unsafe {
                self.tail = (*node).prev;

                match self.tail {
                    None => self.head = None,
                    Some(tail) => (*tail).next = None,
                }

                self.hash_map
                    .remove(&KeyPtr { k: &(*node).key })
                    .map(|node| Box::from_raw(node))
            })
            .flatten()
    }

    #[inline]
    pub fn pop_back(&mut self) -> Option<(K, V)> {
        self.pop_back_node().map(|node| (node.key, node.value))
    }

    #[inline]
    pub fn back(&self) -> Option<(&K, &V)> {
        self.tail
            .map(|node| unsafe { (&(*node).key, &(*node).value) })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.hash_map.len()
    }

    #[inline]
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.hash_map
            .get(Qey::from_ref(key))
            .map(|node| unsafe { &(**node).value })
    }

    #[inline]
    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.hash_map
            .get_mut(Qey::from_ref(key))
            .map(|node| unsafe { &mut (**node).value })
    }

    #[inline]
    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.hash_map.remove(Qey::from_ref(key)).map(|node| unsafe {
            self.remove_node(node);
            let node = Box::from_raw(node);
            (node.key, node.value)
        })
    }

    #[inline]
    pub fn remove_node(&mut self, node: *mut Node<K, V>) {
        unsafe {
            if let Some(head) = self.head {
                if head == node {
                    self.head = (*head).next
                }
            }
            if let Some(tail) = self.tail {
                if tail == node {
                    self.tail = (*tail).prev
                }
            }
            if let Some(next) = (*node).next {
                (*next).prev = (*node).prev
            }
            if let Some(prev) = (*node).prev {
                (*prev).next = (*node).next
            }
        }
    }

    #[inline]
    pub fn take<Q: ?Sized>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.remove(key)
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<(&K, &V)> {
        self.push_back(key, value)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.hash_map.is_empty() && self.head.is_none() && self.tail.is_none()
    }

    #[inline]
    pub fn clear(&mut self) {
        while self.pop_back().is_some() {}
    }

    #[inline]
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            head: self.head.map(|ptr| ptr as *const _),
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn _into_iter(mut self) -> IntoIter<K, V> {
        let head = self.head;
        self.head = None;
        IntoIter {
            head,
            marker: PhantomData,
        }
    }
}

impl<K, V, S> Default for LinkedHashMap<K, V, S>
where
    S: Default,
{
    fn default() -> Self {
        LinkedHashMap {
            hash_map: HashMap::default(),
            head: None,
            tail: None,
            marker: PhantomData,
        }
    }
}

impl<K, V, S> Drop for LinkedHashMap<K, V, S> {
    fn drop(&mut self) {
        unsafe fn drop_node<K, V>(node: *mut Node<K, V>) {
            let node = Box::from_raw(node);
            if let Some(node) = node.next {
                drop_node(node)
            }
        }
        if let Some(node) = self.head {
            unsafe { drop_node(node) }
        }
    }
}

pub struct Iter<'a, K: 'a, V: 'a> {
    head: Option<*const Node<K, V>>,
    marker: PhantomData<(&'a K, &'a V)>,
}

pub struct IntoIter<K, V> {
    head: Option<*mut Node<K, V>>,
    marker: PhantomData<(K, V)>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.head.map(|node| unsafe {
            let kv = (&(*node).key, &(*node).value);
            self.head = (*node).next.map(|ptr| ptr as *const _);
            kv
        })
    }
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.head.map(|node| unsafe {
            self.head = (*node).next;
            let node = Box::from_raw(node);
            (node.key, node.value)
        })
    }
}

impl<K, V> IntoIterator for LinkedHashMap<K, V>
where
    K: Hash + Eq,
{
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self._into_iter()
    }
}

impl<'a, K, V> IntoIterator for &'a LinkedHashMap<K, V>
where
    K: Hash + Eq,
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Iter<'a, K, V> {
        self.iter()
    }
}

impl<K, V> Extend<(K, V)> for LinkedHashMap<K, V>
where
    K: Hash + Eq,
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<K, V> Clone for LinkedHashMap<K, V>
where
    K: Clone + Hash + Eq,
    V: Clone,
{
    fn clone(&self) -> Self {
        let mut map =
            LinkedHashMap::with_capacity_and_hasher(self.len(), self.hash_map.hasher().clone());
        map.extend(self.iter().map(|(k, v)| (k.clone(), v.clone())));
        map
    }
}

impl<K, V> Debug for LinkedHashMap<K, V>
where
    K: Debug + Hash + Eq,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self).finish()
    }
}

impl<K, V> PartialEq for LinkedHashMap<K, V>
where
    K: Hash + Eq,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}

impl<K, V> Eq for LinkedHashMap<K, V>
where
    K: Hash + Eq,
    V: Eq,
{
}

impl<K, V> Hash for LinkedHashMap<K, V>
where
    K: Hash + Eq,
    V: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.iter().for_each(|t| t.hash(state))
    }
}

unsafe impl<K, V> Sync for LinkedHashMap<K, V>
where
    K: Sync,
    V: Sync,
{
}
unsafe impl<K, V> Send for LinkedHashMap<K, V>
where
    K: Send,
    V: Send,
{
}
