use std::ops::Index;

use crate::{arena::ID, solver::table::Table};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Stack {
    stack: Vec<Entry>,
    counter: usize,
}

impl Stack {
    pub fn is_active(&self, table: ID<Table>) -> Option<usize> {
        self.stack.iter().position(|entry| entry.table == table)
    }

    pub fn push(&mut self, table: ID<Table>) -> usize {
        let len = self.stack.len();
        self.stack.push(Entry {
            table,
            depth_first_number: {
                let number = self.counter;
                self.counter += 1;
                DepthFirstNumber(number)
            },
        });

        len
    }

    pub fn pop(&mut self) -> Option<Entry> { self.stack.pop() }
}

impl Index<usize> for Stack {
    type Output = Entry;

    fn index(&self, index: usize) -> &Self::Output {
        self.stack.get(index).expect("Index out of bounds in Stack")
    }
}

impl Stack {
    pub fn new() -> Self { Self { stack: Vec::new(), counter: 0 } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entry {
    pub table: ID<Table>,
    pub depth_first_number: DepthFirstNumber,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DepthFirstNumber(pub usize);

impl DepthFirstNumber {
    pub const MAX: Self = Self(usize::MAX);
}
