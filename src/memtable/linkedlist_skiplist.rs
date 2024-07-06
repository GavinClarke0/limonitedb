use std::cmp::{max};
use std::fmt::Display;
use std::marker::PhantomData;

use std::ptr::{NonNull, null};

use crate::memtable::skiplist::{SkipList, SkipListIterator};

const INSERT_BUFFER_SIZE: usize = 2_usize.pow(2);

pub struct LinkedListSkipList<Key: Ord + Display + Default, const MAX_HEIGHT: usize> where
    Key: Ord,
{
    head: NonNull<Node<Key>>,
    previous: [NonNull<Node<Key>>; MAX_HEIGHT],
    current_height: usize,
    current_size: usize,
    _marker: PhantomData<Key>,
}

impl<Key: Ord + Display + Default, const MAX_HEIGHT: usize> LinkedListSkipList<Key, MAX_HEIGHT> {
    fn new() -> Self {
        Self {
            head: Node::new_head(MAX_HEIGHT),
            previous: std::array::from_fn(|_| NonNull::<Node<Key>>::dangling()),
            current_height: 0,
            current_size: 0,
            _marker: PhantomData,
        }
    }

    // find the node that is closest in value but less then.
    fn find_equal_or_less_then(&self, key: &Key) -> (Link<Key>, [NonNull<Node<Key>>; MAX_HEIGHT]) {
        let mut previous: [NonNull<Node<Key>>; MAX_HEIGHT] = std::array::from_fn(|_| self.head);
        unsafe {
            // 1. Case where node is the smallest or other nodes exist in the tree
            if self.current_height == 0 || (*self.head_next(0).unwrap().as_ptr()).key > *key {
                return (None, previous);
            }
            // 2. Search the rest of the list.
            let mut search_level = self.current_height - 1;
            let mut current_node = self.head;
            loop {
                previous[search_level] = current_node;
                match (*current_node.as_ptr()).next(search_level) {
                    None => {
                        if search_level == 0 {
                            return (None, previous);
                        }
                        search_level -= 1;
                    }
                    Some(next_node) => {
                        if (*next_node.as_ptr()).key >= *key {
                            if (*next_node.as_ptr()).key == *key {
                                return (Some(next_node), previous);
                            } else if search_level == 0 {
                                return (None, previous);
                            }
                            search_level -= 1;
                        } else {
                            current_node = next_node;
                        }
                    }
                };
            }
        }
    }

    #[inline(always)]
    fn get_max_height(&self) -> usize {
        return self.current_height;
    }

    #[inline(always)]
    fn random_height(&self) -> usize {
        let mut height = 1;
        while height < MAX_HEIGHT && fastrand::bool() {
            height += 1
        }
        return height;
    }

    #[inline(always)]
    unsafe fn head_next(&self, level: usize) -> Link<Key> {
        (*self.head.as_ptr()).next(level)
    }

    #[inline(always)]
    unsafe fn head_set_next(&self, level: usize, node: Link<Key>) {
        (*self.head.as_ptr()).set_next(level, node)
    }

    fn print(&self) {
        for i in (0..self.current_height).rev() {
            unsafe {
                let mut next_node = (*self.head.as_ptr()).next(i);
                loop {
                    match next_node {
                        Some(node) => {
                            print!("-> [ {} ]", (*node.as_ptr()).key);
                            next_node = (*node.as_ptr()).next(i);
                        }
                        None => {
                            print!("-> None\n");
                            break;
                        }
                    }
                }
            }
        }
    }
}

impl<Key: Ord + Display + Default, const MAX_HEIGHT: usize> SkipList<Key> for LinkedListSkipList<Key, MAX_HEIGHT>
{
    fn insert(&mut self, key: Key) {
        let (node, previous) = self.find_equal_or_less_then(&key); // This must run so self.previous is populated
        unsafe {
            // 1.
            match node {
                Some(node) => {
                    (*node.as_ptr()).key = key;
                }
                None => {
                    let height = self.random_height();
                    let node = Node::new_link(key, height);
                    for i in 0..height {
                        let previous_node = previous[i].as_ptr();
                        (*node.as_ptr()).set_next(i, (*previous_node).next(i));
                        (*previous_node).set_next(i, Some(node))
                    }
                    self.current_height = max(self.current_height, height);
                    self.current_size += 1
                }
            }
        }
    }

    fn contains(&self, key: &Key) -> bool {
        let (node, _) = self.find_equal_or_less_then(key);
        node.is_some()
    }

    fn estimate_count(&self, _key: &Key) -> usize {
        self.current_size
    }
}

impl<Key: Ord + Display + Default, const MAX_HEIGHT: usize> Drop for LinkedListSkipList<Key, MAX_HEIGHT> {
    fn drop(&mut self) {
        unsafe {
            // Start from the head of the list  // Iterate over each node and deallocate it
            let mut current_node = self.head_next(0);
            while let Some(mut node) = current_node {
                current_node = (*node.as_ptr()).next(0);
                drop(Box::from_raw(node.as_ptr()));
            }
            drop(Box::from_raw(self.head.as_ptr())); // deallocate the head node
        }
    }
}

trait ListNode<Key: Ord> {
    fn set_next(&mut self, n: usize, x: Link<Key>);
    fn next(&self, n: usize) -> Link<Key>;
}


impl<Key: Ord + Default + Display, const MAX_HEIGHT: usize> Iterator for LinkedListSkipList<Key, MAX_HEIGHT>
    where
        Key: Ord,
{
    type Item = Key;

    fn next(&mut self) -> Option<Self::Item> {

        unimplemented!()
    }
}

impl<Key: Ord + Default + Display, const MAX_HEIGHT: usize> SkipListIterator<Key> for LinkedListSkipList<Key, MAX_HEIGHT>
{
    fn valid(&self) -> bool {
        unimplemented!()
    }

    fn key(&self) -> &Key {
        unimplemented!()
    }

    fn next(&mut self) {
        unimplemented!()
    }

    fn prev(&mut self) {
        unimplemented!() // Requires backward links or a stack to track history
    }

    fn seek(&mut self, _target: &Key) {
        unimplemented!()
    }

    fn seek_for_prev(&mut self, _target: &Key) {
        unimplemented!() // Requires backward links or additional tracking
    }

    fn seek_to_first(&mut self) {
        unimplemented!() // Requires access to the list's head
    }

    fn seek_to_last(&mut self) {
        unimplemented!() // Requires full scan or back pointers
    }
}


struct Node<Key: Ord> {
    key: Key,
    links: Vec<Link<Key>>,
}

type Link<Key> = Option<NonNull<Node<Key>>>;

impl<'a, Key: Ord + Default> Node<Key> {
    fn new(key: Key, height: usize) -> Self {
        Self {
            key,
            links: vec![None; height],
        }
    }

    fn new_link(key: Key, height: usize) -> NonNull<Node<Key>> {
        unsafe {
            return NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                key,
                links: vec![None; height],
            })));
        }
    }

    fn new_head(height: usize) -> NonNull<Node<Key>> {
        unsafe {
            return NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                key: Key::default(),
                links: vec![None; height],
            })));
        }
    }

    #[inline(always)]
    fn set_next(&mut self, n: usize, x: Link<Key>) {
        self.links[n] = x;
    }

    #[inline(always)]
    fn next(&self, n: usize) -> Link<Key> {
        debug_assert!(n < self.links.len());
        return self.links[n];
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seq_insert() {
        let mut list: LinkedListSkipList<i32, { 2_usize.pow(6) }> = LinkedListSkipList::new();
        for _i in 0..1000 {
            list.insert(_i);
            assert!(list.contains(&_i));
        }
    }

    #[test]
    fn test_insert_random_insert() {
        let mut list: LinkedListSkipList<i32, { 2_usize.pow(6) }> = LinkedListSkipList::new();
        for _i in 0..1000 {
            let val = fastrand::i32(0..1000);
            list.insert(val);
            assert!(list.contains(&val));
        }
    }
}

