use core::iter;
use core::mem;
use core::option;

use crate::key::Key;
use crate::map::{Entry, MapStorage, OccupiedEntry, VacantEntry};
use crate::option_bucket::{NoneBucket, OptionBucket, SomeBucket};

type Iter<'a, K, V> = iter::Chain<
    iter::Map<
        <<K as Key<'a, V>>::MapStorage as MapStorage<'a, K, V>>::Iter,
        fn((K, &'a V)) -> (Option<K>, &'a V),
    >,
    iter::Map<option::Iter<'a, V>, fn(&'a V) -> (Option<K>, &'a V)>,
>;
type Keys<'a, K, V> = iter::Chain<
    iter::Map<<<K as Key<'a, V>>::MapStorage as MapStorage<'a, K, V>>::Keys, fn(K) -> Option<K>>,
    option::IntoIter<Option<K>>,
>;
type Values<'a, K, V> = iter::Chain<
    <<K as Key<'a, V>>::MapStorage as MapStorage<'a, K, V>>::Values,
    option::Iter<'a, V>,
>;
type IterMut<'a, K, V> = iter::Chain<
    iter::Map<
        <<K as Key<'a, V>>::MapStorage as MapStorage<'a, K, V>>::IterMut,
        fn((K, &'a mut V)) -> (Option<K>, &'a mut V),
    >,
    iter::Map<option::IterMut<'a, V>, fn(&'a mut V) -> (Option<K>, &'a mut V)>,
>;
type ValuesMut<'a, K, V> = iter::Chain<
    <<K as Key<'a, V>>::MapStorage as MapStorage<'a, K, V>>::ValuesMut,
    option::IterMut<'a, V>,
>;
type IntoIter<'a, K, V> = iter::Chain<
    iter::Map<
        <<K as Key<'a, V>>::MapStorage as MapStorage<'a, K, V>>::IntoIter,
        fn((K, V)) -> (Option<K>, V),
    >,
    iter::Map<option::IntoIter<V>, fn(V) -> (Option<K>, V)>,
>;

/// [`MapStorage`] for [`Option`] types.
///
/// # Examples
///
/// ```
/// use fixed_map::{Key, Map};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Key)]
/// enum Part {
///     A,
///     B,
/// }
///
/// #[derive(Debug, Clone, Copy, PartialEq, Key)]
/// enum Key {
///     First(Option<Part>),
///     Second,
/// }
///
/// let mut a = Map::new();
/// a.insert(Key::First(None), 1);
/// a.insert(Key::First(Some(Part::A)), 2);
///
/// assert_eq!(a.get(Key::First(Some(Part::A))), Some(&2));
/// assert_eq!(a.get(Key::First(Some(Part::B))), None);
/// assert_eq!(a.get(Key::First(None)), Some(&1));
/// assert_eq!(a.get(Key::Second), None);
///
/// assert!(a.iter().eq([(Key::First(Some(Part::A)), &2), (Key::First(None), &1)]));
/// assert!(a.values().copied().eq([2, 1]));
/// assert!(a.keys().eq([Key::First(Some(Part::A)), Key::First(None)]));
/// ```
pub struct OptionMapStorage<'a, K, V>
where
    K: Key<'a, V>,
{
    some: K::MapStorage,
    none: Option<V>,
}

impl<'a, K, V> Clone for OptionMapStorage<'a, K, V>
where
    K: Key<'a, V>,
    V: Clone,
    K::MapStorage: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            some: self.some.clone(),
            none: self.none.clone(),
        }
    }
}

impl<'a, K, V> Copy for OptionMapStorage<'a, K, V>
where
    K: Key<'a, V>,
    V: Copy,
    K::MapStorage: Copy,
{
}

impl<'a, K, V> PartialEq for OptionMapStorage<'a, K, V>
where
    K: Key<'a, V>,
    K::MapStorage: PartialEq,
    V: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.none == other.none && self.some == other.some
    }
}

impl<'a, K, V> Eq for OptionMapStorage<'a, K, V>
where
    K: Key<'a, V>,
    K::MapStorage: Eq,
    V: Eq,
{
}

pub enum Vacant<'a, K: 'a, V>
where
    K: Key<'a, V>,
{
    None(NoneBucket<'a, V>),
    Some(<K::MapStorage as MapStorage<'a, K, V>>::Vacant),
}

pub enum Occupied<'a, K: 'a, V>
where
    K: Key<'a, V>,
{
    None(SomeBucket<'a, V>),
    Some(<K::MapStorage as MapStorage<'a, K, V>>::Occupied),
}

impl<'a, K, V> VacantEntry<'a, Option<K>, V> for Vacant<'a, K, V>
where
    K: Key<'a, V>,
{
    #[inline]
    fn key(&self) -> Option<K> {
        match self {
            Vacant::None(_) => None,
            Vacant::Some(entry) => Some(entry.key()),
        }
    }

    #[inline]
    fn insert(self, value: V) -> &'a mut V {
        match self {
            Vacant::None(entry) => entry.insert(value),
            Vacant::Some(entry) => entry.insert(value),
        }
    }
}

impl<'a, K, V> OccupiedEntry<'a, Option<K>, V> for Occupied<'a, K, V>
where
    K: Key<'a, V>,
{
    #[inline]
    fn key(&self) -> Option<K> {
        match self {
            Occupied::None(_) => None,
            Occupied::Some(entry) => Some(entry.key()),
        }
    }

    #[inline]
    fn get(&self) -> &V {
        match self {
            Occupied::None(entry) => entry.as_ref(),
            Occupied::Some(entry) => entry.get(),
        }
    }

    #[inline]
    fn get_mut(&mut self) -> &mut V {
        match self {
            Occupied::None(entry) => entry.as_mut(),
            Occupied::Some(entry) => entry.get_mut(),
        }
    }

    #[inline]
    fn into_mut(self) -> &'a mut V {
        match self {
            Occupied::None(entry) => entry.into_mut(),
            Occupied::Some(entry) => entry.into_mut(),
        }
    }

    #[inline]
    fn insert(&mut self, value: V) -> V {
        match self {
            Occupied::None(entry) => entry.replace(value),
            Occupied::Some(entry) => entry.insert(value),
        }
    }

    #[inline]
    fn remove(self) -> V {
        match self {
            Occupied::None(entry) => entry.take(),
            Occupied::Some(entry) => entry.remove(),
        }
    }
}

impl<'a, K, V: 'a> MapStorage<'a, Option<K>, V> for OptionMapStorage<'a, K, V>
where
    K: Key<'a, V> + 'a,
{
    type Iter = Iter<'a, K, V>;
    type Keys = Keys<'a, K, V>;
    type Values = Values<'a, K, V>;
    type IterMut = IterMut<'a, K, V>;
    type ValuesMut = ValuesMut<'a, K, V>;
    type IntoIter = IntoIter<'a, K, V>;
    type Occupied = Occupied<'a, K, V>;
    type Vacant = Vacant<'a, K, V>;

    #[inline]
    fn empty() -> Self {
        Self {
            some: K::MapStorage::empty(),
            none: Option::default(),
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.some.len() + usize::from(self.none.is_some())
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.some.is_empty() && self.none.is_none()
    }

    #[inline]
    fn insert(&mut self, key: Option<K>, value: V) -> Option<V> {
        match key {
            Some(key) => self.some.insert(key, value),
            None => mem::replace(&mut self.none, Some(value)),
        }
    }

    #[inline]
    fn contains_key(&self, key: Option<K>) -> bool {
        match key {
            Some(key) => self.some.contains_key(key),
            None => self.none.is_some(),
        }
    }

    #[inline]
    fn get(&self, key: Option<K>) -> Option<&V> {
        match key {
            Some(key) => self.some.get(key),
            None => self.none.as_ref(),
        }
    }

    #[inline]
    fn get_mut(&mut self, key: Option<K>) -> Option<&mut V> {
        match key {
            Some(key) => self.some.get_mut(key),
            None => self.none.as_mut(),
        }
    }

    #[inline]
    fn remove(&mut self, key: Option<K>) -> Option<V> {
        match key {
            Some(key) => self.some.remove(key),
            None => mem::replace(&mut self.none, None),
        }
    }

    #[inline]
    fn retain<F>(&mut self, mut func: F)
    where
        F: FnMut(Option<K>, &mut V) -> bool,
    {
        self.some.retain(|k, v| func(Some(k), v));
        if let Some(none) = self.none.as_mut() {
            if !func(None, none) {
                self.none = None;
            }
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.some.clear();
        self.none = None;
    }

    #[inline]
    fn iter(&'a self) -> Self::Iter {
        let map: fn(_) -> _ = |(k, b)| (Some(k), b);
        let a = self.some.iter().map(map);
        let map: fn(_) -> _ = |v| (None, v);
        let b = self.none.iter().map(map);
        a.chain(b)
    }

    #[inline]
    fn keys(&'a self) -> Self::Keys {
        let map: fn(_) -> _ = |k| Some(k);
        self.some
            .keys()
            .map(map)
            .chain(self.none.is_some().then_some(None::<K>))
    }

    #[inline]
    fn values(&'a self) -> Self::Values {
        self.some.values().chain(self.none.iter())
    }

    #[inline]
    fn iter_mut(&'a mut self) -> Self::IterMut {
        let map: fn(_) -> _ = |(k, b)| (Some(k), b);
        let a = self.some.iter_mut().map(map);
        let map: fn(_) -> _ = |v| (None, v);
        let b = self.none.iter_mut().map(map);
        a.chain(b)
    }

    #[inline]
    fn values_mut(&'a mut self) -> Self::ValuesMut {
        self.some.values_mut().chain(self.none.iter_mut())
    }

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let map: fn(_) -> _ = |(k, b)| (Some(k), b);
        let a = self.some.into_iter().map(map);
        let map: fn(_) -> _ = |v| (None, v);
        let b = self.none.into_iter().map(map);
        a.chain(b)
    }

    #[inline]
    fn entry(&'a mut self, key: Option<K>) -> Entry<'a, Self, Option<K>, V> {
        match key {
            Some(key) => match self.some.entry(key) {
                Entry::Occupied(entry) => Entry::Occupied(Occupied::Some(entry)),
                Entry::Vacant(entry) => Entry::Vacant(Vacant::Some(entry)),
            },
            None => match OptionBucket::new(&mut self.none) {
                OptionBucket::Some(some) => Entry::Occupied(Occupied::None(some)),
                OptionBucket::None(none) => Entry::Vacant(Vacant::None(none)),
            },
        }
    }
}
