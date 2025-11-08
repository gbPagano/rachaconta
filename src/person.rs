use std::fmt;
use std::hash::Hash;

use crate::money::Money;

/// Representa um participante na divisão da conta.
///
/// Este enum distingue entre uma pessoa específica, com nome,
/// e um grupo de pessoas anônimas que não fizeram pagamentos.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Person {
    /// Uma pessoa específica que pagou um valor.
    Named { name: String, money_spent: Money },
    /// Um grupo de pessoas que não pagaram.
    /// `size` é o número de pessoas neste grupo (ex: 3 pessoas).
    Unnamed { size: u32 },
}

impl Person {
    pub fn named(name: &str, money_spent: Money) -> Self {
        Person::Named {
            name: name.into(),
            money_spent,
        }
    }

    pub fn unnamed(size: u32) -> Self {
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

    /// Retorna o valor total que esta entidade pagou inicialmente.
    pub fn money_spent(&self) -> Money {
        match self {
            Person::Named { money_spent, .. } => *money_spent,
            Person::Unnamed { .. } => 0.into(),
        }
    }

    /// Retonar quantas pessoas essa entidade representa.
    pub fn size(&self) -> u32 {
        match self {
            Person::Named { .. } => 1,
            Person::Unnamed { size } => *size,
        }
    }
}

impl fmt::Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.identifier())
    }
}
