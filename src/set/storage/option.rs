use core::iter;
use core::mem;
use core::option;

use crate::key::Key;
use crate::set::SetStorage;

type Iter<'a, T> = iter::Chain<
    iter::Map<<<T as Key<'a, ()>>::SetStorage as SetStorage<'a, T>>::Iter, fn(T) -> Option<T>>,
    option::IntoIter<Option<T>>,
>;
type IntoIter<'a, T> = iter::Chain<
    iter::Map<<<T as Key<'a, ()>>::SetStorage as SetStorage<'a, T>>::IntoIter, fn(T) -> Option<T>>,
    option::IntoIter<Option<T>>,
>;

/// [`SetStorage`] for [`Option`] types.
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
pub struct OptionSetStorage<'a, T>
where
    T: Key<'a, ()>,
{
    some: T::SetStorage,
    none: bool,
}

impl<'a, T> Clone for OptionSetStorage<'a, T>
where
    T: Key<'a, ()>,
    T::SetStorage: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            some: self.some.clone(),
            none: self.none,
        }
    }
}

impl<'a, T> Copy for OptionSetStorage<'a, T>
where
    T: Key<'a, ()>,
    T::SetStorage: Copy,
{
}

impl<'a, T> PartialEq for OptionSetStorage<'a, T>
where
    T: Key<'a, ()>,
    T::SetStorage: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.none == other.none && self.some == other.some
    }
}

impl<'a, T> Eq for OptionSetStorage<'a, T>
where
    T: Key<'a, ()>,
    T::SetStorage: Eq,
{
}

impl<'a, T> SetStorage<'a, Option<T>> for OptionSetStorage<'a, T>
where
    T: Key<'a, ()>,
{
    type Iter = Iter<'a, T>;
    type IntoIter = IntoIter<'a, T>;

    #[inline]
    fn empty() -> Self {
        Self {
            some: T::SetStorage::empty(),
            none: false,
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.some.len() + usize::from(self.none)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.some.is_empty() && self.none
    }

    #[inline]
    fn insert(&mut self, value: Option<T>) -> bool {
        match value {
            Some(value) => self.some.insert(value),
            None => mem::replace(&mut self.none, true),
        }
    }

    #[inline]
    fn contains(&self, value: Option<T>) -> bool {
        match value {
            Some(key) => self.some.contains(key),
            None => self.none,
        }
    }

    #[inline]
    fn remove(&mut self, key: Option<T>) -> bool {
        match key {
            Some(key) => self.some.remove(key),
            None => mem::replace(&mut self.none, false),
        }
    }

    #[inline]
    fn retain<F>(&mut self, mut func: F)
    where
        F: FnMut(Option<T>) -> bool,
    {
        self.some.retain(|value| func(Some(value)));

        if self.none {
            self.none = func(None);
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.some.clear();
        self.none = false;
    }

    #[inline]
    fn iter(&'a self) -> Self::Iter {
        let map: fn(_) -> _ = Some;
        self.some
            .iter()
            .map(map)
            .chain(self.none.then_some(None::<T>))
    }

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let map: fn(_) -> _ = Some;
        self.some
            .into_iter()
            .map(map)
            .chain(self.none.then_some(None::<T>))
    }
}
