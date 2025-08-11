use crate::{arena::ID, solver::table::Table};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Stack {
    stack: Vec<Entry>,
}

impl Stack {
    pub fn is_active(&self, table: ID<Table>) -> Option<usize> {
        self.stack.iter().position(|entry| entry.table == table)
    }

    pub fn push(&mut self, table: ID<Table>) {
        self.stack.push(Entry { table });
    }

    pub fn pop(&mut self) -> Option<Entry> { self.stack.pop() }
}

impl Stack {
    pub fn new() -> Self { Self { stack: Vec::new() } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Entry {
    table: ID<Table>,
}
