use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use petgraph::dot::Dot;
use petgraph::prelude::StableDiGraph;
use petgraph::visit::IntoEdgeReferences;
use petgraph::visit::{EdgeRef, IntoNodeReferences, NodeRef};

use crate::money::Money;
use crate::person::Person;

/// Representa uma transação única de pagamento entre duas pessoas.
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
            value,
        }
    }
}

/// Representa o grafo de pagamentos.
pub struct Payments(pub StableDiGraph<Person, Money>);

impl Payments {
    pub fn new(payments: &[Payment]) -> Self {
        let mut graph = StableDiGraph::<Person, Money>::new();

        let persons: HashSet<_> = payments.iter().flat_map(|p| [&p.from, &p.to]).collect();

        let node_map: HashMap<_, _> = persons
            .into_iter()
            .map(|p| {
                let idx = graph.add_node(p.clone());
                (p, idx)
            })
            .collect();

        for p in payments {
            let from = node_map[&p.from];
            let to = node_map[&p.to];

            graph.add_edge(from, to, p.value);
        }

        Self(graph)
    }

    /// Retorna todas as pessoas presentes no grafo.
    pub fn get_persons(&self) -> Vec<Person> {
        self.0
            .node_references()
            .map(|n| n.weight().clone())
            .collect()
    }

    /// Otimiza o grafo de pagamentos para reduzir o número de transações.
    pub fn optimize(&mut self) {
        self.simplify_bidirectional_edges();
    }

    /// Simplifica dívidas mútuas entre duas pessoas.
    ///
    /// Mantém apenas o saldo líquido: por exemplo, se A deve 10 para B, e B deve 7 para A,
    /// o resultado será A deve 3 para B. Se forem iguais, ambas são removidas.
    fn simplify_bidirectional_edges(&mut self) {
        let indexes = self.0.edge_indices().collect::<Vec<_>>();
        for edge in indexes {
            if let Some((source, target)) = self.0.edge_endpoints(edge) {
                if let Some(e2) = self.0.find_edge(target, source)
                    && let Some(e1) = self.0.find_edge(source, target)
                {
                    let w1 = self.0.edge_weight(e1).unwrap();
                    let w2 = self.0.edge_weight(e2).unwrap();

                    match w1.cmp(w2) {
                        Ordering::Less => {
                            // Aresta A -> B é removida
                            // Aresta B -> A é atualizada com a diferença
                            self.0.update_edge(target, source, *w2 - *w1);
                            self.0.remove_edge(e1);
                        }
                        Ordering::Greater => {
                            // Aresta A -> B é atualizada com a diferença
                            // Aresta B -> A é removida
                            self.0.update_edge(source, target, *w1 - *w2);
                            self.0.remove_edge(e2);
                        }
                        Ordering::Equal => {
                            // Dívidas se anulam
                            self.0.remove_edge(e1);
                            self.0.remove_edge(e2);
                        }
                    }
                }
            }
        }
    }

    pub fn to_vec(&self) -> Vec<Payment> {
        let persons = self.get_persons();
        self.0
            .edge_references()
            .map(|edge| {
                let source = persons
                    .iter()
                    .find(|p| p == &self.0.node_weight(edge.source()).unwrap())
                    .unwrap();
                let target = persons
                    .iter()
                    .find(|p| p == &self.0.node_weight(edge.target()).unwrap())
                    .unwrap();
                Payment::new(source, target, *edge.weight())
            })
            .collect()
    }

    /// Imprime a representação do grafo no formato Graphviz DOT na saída padrão.
    pub fn print_dot(&self) {
        let dot = Dot::new(&self.0);
        println!("{dot}");
    }
}

impl FromIterator<Person> for Payments {
    fn from_iter<T: IntoIterator<Item = Person>>(iter: T) -> Self {
        let persons: Vec<Person> = iter.into_iter().collect();
        let mut payments = Vec::new();

        let num_persons: usize = persons
            .iter()
            .map(|p| match p {
                Person::Named { .. } => 1,
                Person::Unnamed { size } => *size,
            })
            .sum();

        for creditor in persons.iter() {
            if matches!(creditor, Person::Unnamed { .. })
                || matches!(creditor, Person::Named { money_spent, .. } if money_spent.cents() == 0)
            {
                continue;
            }

            let amount_for_each = creditor.money_spent() / num_persons as f64;
            for debitor in persons.iter().filter(|p| p != &creditor) {
                let amount = match debitor {
                    Person::Named { .. } => amount_for_each,
                    Person::Unnamed { size } => amount_for_each * *size as f64,
                };

                payments.push(Payment::new(debitor, creditor, amount));
            }
        }

        Payments::new(&payments)
    }
}
