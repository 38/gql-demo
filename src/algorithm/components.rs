use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::iter::Enumerate;
use std::fmt::{Debug, Result, Formatter};

use crate::properties::WithRegion;

pub struct Point<T: WithRegion> {
    pub is_open: bool,
    pub index: usize,
    pub depth: usize,
    pub value: T,
}

impl <T: WithRegion> Debug for Point<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
       if self.is_open {
           write!(f, "Open(")?;
       } else {
           write!(f, "Close(")?;
       }

       let (chrom, pos) = self.position();

       write!(f, "{}, {}, {})", chrom, pos, self.depth)
    }
}

impl <T: WithRegion> Point<T> {
    pub fn position(&self) -> (&str, u32) {
        if self.is_open {
            (self.value.chrom(), self.value.begin())
        } else {
            (self.value.chrom(), self.value.end())
        }
    }
}

impl <T: WithRegion> PartialEq for Point<T> {
    fn eq(&self, other: &Point<T>) -> bool {
        self.position() == other.position()
    }
}

impl <T: WithRegion> PartialOrd for Point<T> {
    fn partial_cmp(&self, other: &Point<T>) -> Option<Ordering> {
        let ret = self.position().cmp(&other.position())
            .then_with(|| self.is_open.cmp(&other.is_open));
        Some(ret)
    }
}

impl <T: WithRegion> Eq for Point<T> {}

impl <T: WithRegion> Ord for Point<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub struct ComponentsIter<I>
where 
    I: Iterator,
    I::Item : WithRegion + Clone
{
    iter: Enumerate<I>,
    peek_buffer: Option<(usize, I::Item)>,
    heap: BinaryHeap<Reverse<Point<I::Item>>>,
}

pub trait Components
where
    Self: Iterator + Sized,
    Self::Item : WithRegion + Clone 
{
    fn components(self) -> ComponentsIter<Self> {
        let mut iter = self.enumerate();
        let peek_buffer = iter.next(); 
        ComponentsIter {
            iter,
            peek_buffer,
            heap: BinaryHeap::new()
        }
    }
}

impl <T> Components for T 
where
    T: Iterator + Sized,
    Self::Item : WithRegion + Clone,
{}


impl <I> Iterator for ComponentsIter<I>
where 
    I: Iterator,
    I::Item : WithRegion + Clone
{
    type Item = Point<I::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((index, peek_buffer)) = self.peek_buffer.as_ref() {
            let index = *index;
            if self.heap.peek().map_or(false, |x| x.0.position() < (peek_buffer.chrom(), peek_buffer.begin())) {
                let depth = self.heap.len();
                return self.heap.pop().map(|Reverse(mut x)| {
                    x.depth = depth - 1;
                    x
                });
            }
            let depth = self.heap.len() + 1;

            self.heap.push(Reverse(Point {
                index,
                depth: 0,
                value: peek_buffer.clone(),
                is_open: false,
            }));
            let ret = Some(Point {
                index,
                depth,
                is_open: true,
                value: peek_buffer.clone(),
            });
            self.peek_buffer = self.iter.next();
            ret
        } else {
            let depth = self.heap.len();
            self.heap.pop().map(|Reverse(mut x)| {
                x.depth = depth - 1;
                x
            })
        }
    }
}
