use std::cmp::Ordering;
use std::marker::PhantomData;


pub trait SkipList<Key: Ord> {

    /// Inserts a key into the list.
    /// Requires that nothing which compares equal to `key` is currently in the list.
    fn insert(&mut self, key: Key);

    /// Returns true if an entry that compares equal to `key` is in the list.
    fn contains(&self, key: &Key) -> bool;

    /// Returns the estimated number of entries smaller than `key`.
    fn estimate_count(&self, key: &Key) -> u64;
}

pub trait SkipListIterator<Key>: Iterator<Item = Key> {
    /// Returns true if the iterator is positioned at a valid node.
    fn valid(&self) -> bool;

    /// Returns the key at the current position.
    /// Requires that the iterator is valid.
    fn key(&self) -> &Key;

    /// Advances to the next position.
    /// Requires that the iterator is valid.
    fn next(&mut self);

    /// Advances to the previous position.
    /// Requires that the iterator is valid.
    fn prev(&mut self);

    /// Advance to the first entry with a key >= target.
    fn seek(&mut self, target: &Key);

    /// Retreat to the last entry with a key <= target.
    fn seek_for_prev(&mut self, target: &Key);

    /// Position at the first entry in list.
    /// Final state of iterator is Valid() iff list is not empty.
    fn seek_to_first(&mut self);

    /// Position at the last entry in list.
    /// Final state of iterator is Valid() iff list is not empty.
    fn seek_to_last(&mut self);
}
