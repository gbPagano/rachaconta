use std::collections::{BinaryHeap, HashMap, HashSet};

use petgraph::dot::Dot;
use petgraph::prelude::StableDiGraph;
use petgraph::visit::IntoEdgeReferences;
use petgraph::visit::{EdgeRef, IntoNodeReferences, NodeRef};

use crate::money::Money;
use crate::person::Person;

/// Representa uma transação única de pagamento entre duas pessoas.
#[derive(Debug, PartialEq, Eq, Hash)]
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
pub struct Payments(StableDiGraph<Person, Money>);

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
    ///
    /// Esta função modifica o grafo de pagamentos para reduzir o número de
    /// pagamentos necessários, mantendo o o mesmo resultado financeiro final.
    ///
    /// O algoritmo segue quatro etapas principais:
    /// 1. **Cálculo dos balanços líquidos** — soma tudo o que cada pessoa deve e tem a receber,
    ///    resultando em um valor líquido positivo (credor) ou negativo (devedor).
    /// 2. **Separação de devedores e credores** — organiza ambos os grupos em *heaps*,
    ///    priorizando os maiores valores para otimizar as quitações.
    /// 3. **Quitação de dívidas** — faz as compensações diretas entre os maiores devedores
    ///    e credores até que todos os balanços sejam zerados.
    /// 4. **Atualizar Grafo:** Remove todas as arestas de pagamento originais e, em seguida,
    ///    insere as novas arestas otimizada.
    pub fn optimize(&mut self) {
        // 1. Calcular Balanços Líquidos
        // `i64` é usado para permitir balanços negativos (devedores).
        let mut balances = HashMap::<Person, i64>::new();
        for person in self.get_persons() {
            balances.insert(person, 0);
        }

        for edge in self.0.edge_references() {
            let value = edge.weight().raw() as i64;
            let source = self.0.node_weight(edge.source()).unwrap();
            let target = self.0.node_weight(edge.target()).unwrap();

            // `source` (pagador) tem seu balanço diminuído
            *balances.entry(source.clone()).or_default() -= value;
            // `target` (recebedor) tem seu balanço aumentado
            *balances.entry(target.clone()).or_default() += value;
        }

        // 2. Separar Devedores e Credores
        let mut debtors = BinaryHeap::new();
        let mut creditors = BinaryHeap::new();

        for (node_idx, balance) in balances {
            match balance {
                b if b < 0 => debtors.push((-b, node_idx)),
                b if b > 0 => creditors.push((b, node_idx)),
                _ => (),
            }
        }

        // 3. Quitar as Dívidas
        let mut new_payments = Vec::new();

        // Pega o maior devedor e o maior credor
        while let (Some(mut debtor_entry), Some(mut creditor_entry)) =
            (debtors.pop(), creditors.pop())
        {
            let debt = debtor_entry.0;
            let credit = creditor_entry.0;

            let transfer_amount = debt.min(credit);

            new_payments.push(Payment::new(
                &debtor_entry.1,
                &creditor_entry.1,
                Money::from(transfer_amount as f64 / 1000.),
            ));

            let remaining_debt = debt - transfer_amount;
            let remaining_credit = credit - transfer_amount;

            // Se o devedor ainda deve algo, ele volta para o heap
            if remaining_debt > 0 {
                debtor_entry.0 = remaining_debt;
                debtors.push(debtor_entry);
            }

            // Se o credor ainda tem algo a receber, ele volta para o heap
            if remaining_credit > 0 {
                creditor_entry.0 = remaining_credit;
                creditors.push(creditor_entry);
            }
        }
        // 4. Atualizar grafo
        // Mapeamos os nodes ja existentes no grafo.
        let node_map: HashMap<_, _> = self
            .0
            .node_references()
            .map(|(i, p)| (p.clone(), i))
            .collect();

        // Limpamos as edges atuais, e adicionamos os novos pagamentos otimizados
        self.0.clear_edges();
        for p in new_payments.into_iter() {
            let from = node_map[&p.from];
            let to = node_map[&p.to];

            self.0.add_edge(from, to, p.value);
        }

        // Garante que o balanço final continua correto
        debug_assert!(self.validate());
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

    /// Verifica se os pagamentos estão consistentes dentro de um limite de tolerância.
    ///
    /// Calcula o valor médio que cada pessoa deveria ter pago e compara com o saldo
    /// final de cada participante (considerando o que gastou, pagou e recebeu).
    ///
    /// Aceita pequenas diferenças de até '0,05 centavo * número de participantes'.
    /// Retorna `true` se todos os saldos estiverem dentro desse limite.
    pub fn validate(&self) -> bool {
        let payments = self.to_vec();
        let persons = self.get_persons();

        let num_persons: u32 = persons.iter().map(|p| p.size()).sum();
        let total_debt: Money = persons.iter().map(|p| p.money_spent()).sum();
        let amount_for_each = total_debt / num_persons;

        for person in persons {
            let to_receive: Money = payments
                .iter()
                .filter(|p| p.to == person)
                .map(|p| p.value)
                .sum();
            let to_pay: Money = payments
                .iter()
                .filter(|p| p.from == person)
                .map(|p| p.value)
                .sum();

            let final_balance = (person.money_spent() + to_pay - to_receive) / person.size();

            // Verifica se a diferença está dentro do limite de tolerância.
            // O limite máximo é calculado como 0.05 centavos multiplicado pelo
            // número total de pessoas, garantindo uma margem proporcional ao grupo.
            // Há também um limite mínimo de 0.01 centavo para evitar falsos positivos
            // em grupos muito pequenos.
            let diff = (amount_for_each.decimal() - final_balance.decimal()).abs();
            let max_diff = (0.0005 * num_persons as f64).max(0.01);
            if diff.round() >= max_diff {
                dbg!(diff, max_diff, amount_for_each, final_balance);
                return false;
            }
        }
        true
    }
}

impl FromIterator<Person> for Payments {
    fn from_iter<T: IntoIterator<Item = Person>>(iter: T) -> Self {
        let persons: Vec<Person> = iter.into_iter().collect();
        let mut payments = Vec::new();

        let num_persons: u32 = persons.iter().map(|p| p.size()).sum();

        for creditor in persons.iter() {
            if creditor.money_spent() == Money::from(0) {
                continue;
            }

            let amount_for_each = creditor.money_spent() / num_persons as f64;
            for debitor in persons.iter().filter(|p| p != &creditor) {
                let amount = amount_for_each * debitor.size();

                payments.push(Payment::new(debitor, creditor, amount));
            }
        }

        Payments::new(&payments)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    // #[test]
    // fn simplify_bidirectional_edges() {
    //     let persons = vec![
    //         Person::named("A", 10.into()),
    //         Person::named("B", 20.into()),
    //         Person::named("C", 10.into()),
    //         Person::unnamed(1),
    //     ];
    //
    //     let mut initial_payments: Payments = persons.clone().into_iter().collect();
    //
    //     let final_payments = vec![
    //         Payment::new(&persons[0], &persons[1], 2.5.into()),
    //         Payment::new(&persons[2], &persons[1], 2.5.into()),
    //         Payment::new(&persons[3], &persons[0], 2.5.into()),
    //         Payment::new(&persons[3], &persons[2], 2.5.into()),
    //         Payment::new(&persons[3], &persons[1], 5.into()),
    //     ];
    //
    //     initial_payments.simplify_bidirectional_edges();
    //     let left: HashSet<Payment> = HashSet::from_iter(initial_payments.to_vec());
    //     let right: HashSet<Payment> = HashSet::from_iter(final_payments);
    //
    //     assert_eq!(left, right);
    //     assert!(initial_payments.validate());
    // }
    //
    // #[test]
    // fn simplify_transitive_edges() {
    //     let persons = vec![
    //         Person::named("A", 14.into()),
    //         Person::named("B", 20.into()),
    //         Person::named("C", 8.into()),
    //         Person::unnamed(1),
    //     ];
    //
    //     let mut initial_payments: Payments = persons.clone().into_iter().collect();
    //
    //     let final_payments = vec![
    //         Payment::new(&persons[2], &persons[1], 2.5.into()),
    //         Payment::new(&persons[3], &persons[1], 7.into()),
    //         Payment::new(&persons[3], &persons[0], 3.5.into()),
    //     ];
    //
    //     initial_payments.simplify_bidirectional_edges();
    //     initial_payments.simplify_transitive_edges();
    //     let left: HashSet<Payment> = HashSet::from_iter(initial_payments.to_vec());
    //     let right: HashSet<Payment> = HashSet::from_iter(final_payments);
    //
    //     assert_eq!(left, right);
    //     assert!(initial_payments.validate());
    // }
}
