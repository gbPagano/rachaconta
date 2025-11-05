use std::hash::Hash;

use crate::money::Money;

#[derive(Debug, Clone, PartialEq)]
pub enum Person {
    Named { name: String, money_spent: Money },
    Unnamed { size: usize },
}

impl Person {
    pub fn named(name: &str, money_spent: Money) -> Self {
        Person::Named {
            name: name.into(),
            money_spent,
        }
    }

    pub fn unnamed(size: usize) -> Self {
        Person::Unnamed { size }
    }

    pub fn identifier(&self) -> String {
        match self {
            Person::Named {
                name,
                money_spent: _,
            } => name.clone(),
            Person::Unnamed { size } => format!("Outras {size} pessoas"),
        }
    }

    pub fn money_spent(&self) -> Money {
        match self {
            Person::Named { money_spent, .. } => *money_spent,
            Person::Unnamed { .. } => 0.into(),
        }
    }
}

impl Eq for Person {}

impl Hash for Person {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.identifier().hash(state);
    }
}
