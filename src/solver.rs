use std::cmp::Ordering;
use std::collections::HashSet;

use petgraph::dot::Dot;
use petgraph::graph::DiGraph;
use petgraph::visit::EdgeRef;

use crate::money::Money;
use crate::payment::{Payment, ToGraph};
use crate::person::Person;

pub fn calc_payments(persons: &[Person]) -> Vec<Payment> {
    let mut payments = Vec::new();

    let num_persons: usize = persons
        .iter()
        .map(|p| match p {
            Person::Named { .. } => 1,
            Person::Unnamed { size } => *size,
        })
        .sum();

    for creditor in persons {
        if matches!(creditor, Person::Unnamed { .. })
            || matches!(creditor, Person::Named { money_spent, .. } if *money_spent == 0.into())
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

    payments
}

pub fn optimize_payments(payments: &[Payment]) -> Vec<Payment> {
    let mut graph = payments.to_graph();

    simplify_bidirectional_edges(&mut graph);
    // let dot = Dot::new(&graph);
    // println!("{dot:?}");

    let persons: HashSet<_> = payments.iter().flat_map(|p| [&p.from, &p.to]).collect();

    graph
        .edge_references()
        .map(|edge| {
            let source = persons
                .iter()
                .find(|p| &p.identifier() == graph.node_weight(edge.source()).unwrap())
                .unwrap();
            let target = persons
                .iter()
                .find(|p| &p.identifier() == graph.node_weight(edge.target()).unwrap())
                .unwrap();
            Payment::new(source, target, *edge.weight())
        })
        .collect()
}

fn simplify_bidirectional_edges(graph: &mut DiGraph<String, Money>) {
    for edge in graph.edge_indices() {
        if let Some((source, target)) = graph.edge_endpoints(edge) {
            if let Some(e2) = graph.find_edge(target, source)
                && let Some(e1) = graph.find_edge(source, target)
            {
                let w1 = graph.edge_weight(e1).unwrap();
                let w2 = graph.edge_weight(e2).unwrap();

                match w1.cmp(&w2) {
                    Ordering::Less => {
                        graph.update_edge(target, source, *w2 - *w1);
                        graph.remove_edge(e1);
                    }
                    Ordering::Greater => {
                        graph.update_edge(source, target, *w1 - *w2);
                        graph.remove_edge(e2);
                    }
                    Ordering::Equal => {
                        graph.remove_edge(e1);
                        graph.remove_edge(e2);
                    }
                }
            }
        }
    }
}
