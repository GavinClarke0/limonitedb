
use std::cmp::{max};
use std::fmt::Display;
use std::marker::PhantomData;

use std::ptr::NonNull;

use crate::memtable::skiplist::{SkipList, SkipListIterator};

const INSERT_BUFFER_SIZE: usize = 2_usize.pow(2);

pub struct LinkedListSkipList<Key, const MAX_HEIGHT: usize> where
    Key: Ord,
{
    head: Link<Key>,
    previous: [Link<Key>; MAX_HEIGHT],
    current_height: usize,
    _marker: PhantomData<Key>,
}

impl <Key: Ord + Display, const MAX_HEIGHT: usize> LinkedListSkipList<Key, MAX_HEIGHT> {
    fn new() -> Self {
        Self {
            head: None,
            previous: std::array::from_fn(|_| None),
            current_height: 0,
            _marker: PhantomData,
        }
    }


    // find the node that is closest in value but less then.
    fn find_less_then(&mut self, key: &Key) -> Link<Key> {
        let mut search_level = match self.current_height{
            0 => {return None}
            height => { height - 1}
        };

        debug_assert!(self.head.is_some()); // if height is one this should always be present.
        let mut current_node = self.head.unwrap();
        unsafe {
            // 1. Special case compare with first node to determine if value is new head.
            if (*current_node.as_ptr()).key >= *key {
                return None
            }
            // 2. search rest of skip list.
            loop {
                match (*current_node.as_ptr()).next(search_level) {
                    None => {
                        self.previous[search_level] = Some(current_node);
                        if search_level == 0 {
                            return Some(current_node);
                        } else {
                            search_level -= 1;
                        }
                    }
                    Some(next_node) => {
                        self.previous[search_level] = Some(current_node);
                        if (*next_node.as_ptr()).key >= *key {
                            if search_level == 0 {
                                return Some(current_node);
                            } else {
                                search_level -= 1;
                            }
                        } else {
                            current_node = next_node;
                        }
                    }
                };
            }
        }
    }

    // find the node that is closest in value but less then.
    fn find_node(&self, key: &Key) -> Link<Key> {
        let mut search_level = match self.current_height{
            0 => {return None}
            height => { height - 1}
        };
        debug_assert!(self.head.is_some()); // if height is one this should always be present.
        let mut current_node = self.head.unwrap();
        unsafe {

            // 1. check if first node is value, if it is less we can exit early as the key
            // cannot exist in the list.
            if (*current_node.as_ptr()).key == *key {
                return Some(current_node)
            } else if (*current_node.as_ptr()).key > *key {
                return None
            }
            // 2. search the rest of the list for the values in the list.
            loop {
                match (*current_node.as_ptr()).next(search_level) {
                    None => {
                        if search_level == 0 {
                            return None;
                        } else {
                            search_level -= 1;
                        }
                    }
                    Some(next_node) => {
                        if (*next_node.as_ptr()).key > *key {
                            if search_level == 0 {
                                return None;
                            } else {
                                search_level -= 1;
                            }
                        } else if (*next_node.as_ptr()).key == *key{
                            return Some(next_node);
                        } else {
                            current_node = next_node;
                        }
                    }
                };
            }
        }
    }

    fn get_max_height(&self) -> usize {
        return self.current_height
    }

    fn random_height(&self) -> usize {
        let mut height = 1;
        while height < MAX_HEIGHT && fastrand::bool() {
            height += 1
        }
        return height
    }

     fn print(&self) {

        for i in (0..self.current_height).rev() {
            unsafe {
                print!("[ {} ]", (*self.head.unwrap().as_ptr()).key);
                let mut next_node = (*self.head.unwrap().as_ptr()).links[i];
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

impl<Key: Ord + Display, const MAX_HEIGHT: usize> SkipList<Key> for LinkedListSkipList< Key, MAX_HEIGHT >
{
    fn insert(&mut self, key: Key) {
        let height = self.random_height();
        let max_height = max(self.current_height, height);
        let insert_after_node = self.find_less_then(&key);
        unsafe {
            if let Some(_) = insert_after_node {
                // Case where list is not empty.
                let node = Node::new_link(key, height);
                for i in 0..self.current_height {
                    let previous = self.previous[i].unwrap().as_ptr();
                    (*node.as_ptr()).set_next(i, (*previous).next(i));
                    (*previous).set_next(i, Some(node))
                }
                for i in self.current_height..max_height {
                    (*self.head.unwrap().as_ptr()).set_next(i, Some(node));
                    (*node.as_ptr()).set_next(i, None);
                }

            } else {
                // Case where list is empty or value is smallest in list.
                let node = Node::new_link(key, max_height);
                for i in 0..self.current_height {
                    (*node.as_ptr()).set_next(i, (*self.head.unwrap().as_ptr()).next(i));
                }
                for i in self.current_height..max_height {
                    (*node.as_ptr()).set_next(i, None);
                }
                self.head = Some(node)
            }
        }
        self.current_height = max_height;
    }

    fn contains(&self, key: &Key) -> bool {
        return self.find_node(key).is_some();
    }

    fn estimate_count(&self, _key: &Key) -> u64 {
        todo!()
    }
}


impl<Key, const MAX_HEIGHT: usize> Iterator for LinkedListSkipList< Key, MAX_HEIGHT >
    where
        Key: Ord,
{
    type Item = Key;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

impl<Key: Ord, const MAX_HEIGHT: usize> SkipListIterator<Key> for LinkedListSkipList<Key, MAX_HEIGHT>
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
    links: Vec<Link<Key>>
}
type Link<Key> =  Option<NonNull<Node<Key>>>;

impl<'a, Key: Ord> Node<Key> {
    fn new(key: Key, height: usize) -> Self{
        Self{
            key,
            links: vec![None; height]
        }
    }

    fn new_link(key: Key, height: usize) -> NonNull<Node<Key>> {
        unsafe {
            return NonNull::new_unchecked(Box::into_raw(Box::new( Node{
                key,
                links: vec![None; height]
            })))
        }
    }

    #[inline(always)]
    fn set_next(&mut self, n: usize, x: Link<Key>) {
        if n >= self.links.len() {
            self.links.resize(n + INSERT_BUFFER_SIZE, None)
        }
        self.links[n] = x;
    }

    #[inline(always)]
    fn next(&self, n: usize) -> Link<Key> {
        debug_assert!(n < self.links.len());
        return self.links[n]
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_contains() {
        let mut list: LinkedListSkipList<i32, {2^12}> = LinkedListSkipList::new();

        for _i in 0..100 {
            list.insert(fastrand::i32(0..100000));
        }
        list.print();

        for i in 0..50000 {
            assert!(list.contains(&i));
        }
    }
}

