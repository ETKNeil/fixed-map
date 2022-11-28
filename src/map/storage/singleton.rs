use core::mem;

use crate::map::{Entry, MapStorage};
use crate::option_bucket::{NoneBucket, OptionBucket, SomeBucket};

/// [`MapStorage`] type that can only inhabit a single value (like `()`).
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SingletonMapStorage<V> {
    inner: Option<V>,
}

impl<V> PartialEq for SingletonMapStorage<V>
where
    V: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<V> Eq for SingletonMapStorage<V> where V: Eq {}

impl<'a, K, V: 'a> MapStorage<'a, K, V> for SingletonMapStorage<V>
where
    K: Default + 'a,
{
    type Iter = ::core::option::IntoIter<(K, &'a V)>;
    type Keys = ::core::option::IntoIter<K>;
    type Values = ::core::option::Iter<'a, V>;
    type IterMut = ::core::option::IntoIter<(K, &'a mut V)>;
    type ValuesMut = ::core::option::IterMut<'a, V>;
    type IntoIter = ::core::option::IntoIter<(K, V)>;
    type Occupied = SomeBucket<'a, V>;
    type Vacant = NoneBucket<'a, V>;

    #[inline]
    fn empty() -> Self {
        Self {
            inner: Option::default(),
        }
    }

    #[inline]
    fn len(&self) -> usize {
        usize::from(self.inner.is_some())
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.inner.is_none()
    }

    #[inline]
    fn insert(&mut self, _: K, value: V) -> Option<V> {
        mem::replace(&mut self.inner, Some(value))
    }

    #[inline]
    fn contains_key(&self, _: K) -> bool {
        self.inner.is_some()
    }

    #[inline]
    fn get(&self, _: K) -> Option<&V> {
        self.inner.as_ref()
    }

    #[inline]
    fn get_mut(&mut self, _: K) -> Option<&mut V> {
        self.inner.as_mut()
    }

    #[inline]
    fn remove(&mut self, _: K) -> Option<V> {
        mem::replace(&mut self.inner, None)
    }

    #[inline]
    fn retain<F>(&mut self, mut func: F)
    where
        F: FnMut(K, &mut V) -> bool,
    {
        if let Some(val) = self.inner.as_mut() {
            if !func(K::default(), val) {
                self.inner = None;
            }
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.inner = None;
    }

    #[inline]
    fn iter(&'a self) -> Self::Iter {
        self.inner.as_ref().map(|v| (K::default(), v)).into_iter()
    }

    #[inline]
    fn keys(&'a self) -> Self::Keys {
        Some(K::default()).into_iter()
    }

    #[inline]
    fn values(&'a self) -> Self::Values {
        self.inner.iter()
    }

    #[inline]
    fn iter_mut(&'a mut self) -> Self::IterMut {
        self.inner.as_mut().map(|v| (K::default(), v)).into_iter()
    }

    #[inline]
    fn values_mut(&'a mut self) -> Self::ValuesMut {
        self.inner.iter_mut()
    }

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.map(|v| (K::default(), v)).into_iter()
    }

    #[inline]
    fn entry(&'a mut self, _key: K) -> Entry<'a, Self, K, V> {
        match OptionBucket::new(&mut self.inner) {
            OptionBucket::Some(some) => Entry::Occupied(some),
            OptionBucket::None(none) => Entry::Vacant(none),
        }
    }
}
