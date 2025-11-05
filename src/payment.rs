use std::collections::{HashMap, HashSet};

use petgraph::dot::{Config, Dot};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;

use crate::money::Money;
use crate::person::Person;

#[derive(Debug)]
pub struct Payment {
    pub from: Person,
    pub to: Person,
    pub value: Money,
}

impl Payment {
    pub fn new(from: &Person, to: &Person, value: Money) -> Self {
        Self {
            from: from.clone(),
            to: to.clone(),
            value: value.into(),
        }
    }
}

pub trait ToGraph {
    fn to_graph(self) -> DiGraph<String, Money>;
}

impl ToGraph for &[Payment] {
    fn to_graph(self) -> DiGraph<String, Money> {
        let mut graph = DiGraph::<String, Money>::new();

        let persons: HashSet<_> = self.iter().flat_map(|p| [&p.from, &p.to]).collect();

        let node_map: HashMap<_, _> = persons
            .into_iter()
            .map(|p| {
                let idx = graph.add_node(p.identifier());
                (p, idx)
            })
            .collect();

        for payment in self {
            let from = node_map[&payment.from];
            let to = node_map[&payment.to];

            graph.add_edge(from, to, payment.value);
        }

        graph
    }
}
