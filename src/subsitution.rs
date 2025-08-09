use std::collections::HashMap;

use crate::term::Term;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Substitution {
    pub mapping: HashMap<String, Term>,
}
