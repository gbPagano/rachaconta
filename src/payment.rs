use std::collections::{BinaryHeap, HashMap, HashSet};

use petgraph::dot::Dot;
use petgraph::prelude::StableDiGraph;
use petgraph::visit::IntoEdgeReferences;
use petgraph::visit::{EdgeRef, IntoNodeReferences, NodeRef};
use rust_decimal::{Decimal, dec};

use crate::person::Person;

/// Representa uma transação única de pagamento entre duas pessoas.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Payment {
    pub from: Person,
    pub to: Person,
    pub value: Decimal,
}

impl Payment {
    pub fn new(from: &Person, to: &Person, value: Decimal) -> Self {
        Self {
            from: from.clone(),
            to: to.clone(),
            value,
        }
    }
}

/// Representa o grafo de pagamentos.
pub struct Payments(StableDiGraph<Person, Decimal>);

impl Payments {
    pub fn new(payments: &[Payment]) -> Self {
        let mut graph = StableDiGraph::<Person, Decimal>::new();

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
        let mut balances = HashMap::<Person, Decimal>::new();
        for person in self.get_persons() {
            balances.insert(person, 0.into());
        }

        for edge in self.0.edge_references() {
            let value = edge.weight();
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
                b if b.is_sign_negative() => debtors.push((-b, node_idx)),
                b if b.is_sign_positive() => creditors.push((b, node_idx)),
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

            let transfer_amount = debt.min(credit).round_dp(2);

            new_payments.push(Payment::new(
                &debtor_entry.1,
                &creditor_entry.1,
                transfer_amount,
            ));

            let remaining_debt = (debt - transfer_amount).round_dp(2);
            let remaining_credit = (credit - transfer_amount).round_dp(2);

            // Se o devedor ainda deve algo, ele volta para o heap
            if remaining_debt > dec!(0.01) {
                debtor_entry.0 = remaining_debt;
                debtors.push(debtor_entry);
            }

            // Se o credor ainda tem algo a receber, ele volta para o heap
            if remaining_credit > dec!(0.01) {
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
    /// Aceita pequenas diferenças de até 2 centavos.
    /// Retorna `true` se todos os saldos estiverem dentro desse limite.
    pub fn validate(&self) -> bool {
        let payments = self.to_vec();
        let persons = self.get_persons();

        let num_persons: Decimal = persons.iter().map(|p| Decimal::from(p.size())).sum();
        let total_debt: Decimal = persons.iter().map(|p| p.money_spent()).sum();
        let amount_for_each = total_debt / num_persons;

        for person in persons {
            let to_receive: Decimal = payments
                .iter()
                .filter(|p| p.to == person)
                .map(|p| p.value)
                .sum();
            let to_pay: Decimal = payments
                .iter()
                .filter(|p| p.from == person)
                .map(|p| p.value)
                .sum();

            let final_balance =
                (person.money_spent() + to_pay - to_receive) / Decimal::from(person.size());

            // Verifica se a diferença está dentro do limite de tolerância.
            // O limite máximo é de 2 centavos
            let diff = (amount_for_each - final_balance).abs();
            if diff.round_dp(2) > dec!(0.02) {
                dbg!(diff, amount_for_each, final_balance, person);
                return false;
            }
        }
        true
    }

    /// Imprime no console um relatório textual e legível da divisão de contas.
    pub fn print_text(&self) {
        let payments = self.to_vec();

        // Ordenamos as pessoas para que as Unnamed fiquem por último.
        let mut persons = self.get_persons();
        persons.sort();

        let num_persons: u32 = persons.iter().map(|p| p.size()).sum();
        let total_amount: Decimal = persons.iter().map(|p| p.money_spent()).sum();
        let amount_for_each: Decimal = total_amount / Decimal::from(num_persons);

        println!("Valor total da conta: {total_amount:.2}");
        println!("    {amount_for_each:.2} para cada");

        for person in persons {
            let debts: Vec<_> = payments.iter().filter(|p| p.from == person).collect();

            let mut total_debt: Decimal = debts.iter().map(|p| p.value).sum();
            if let Person::Unnamed { size } = person {
                total_debt /= Decimal::from(size);
            }

            let total_to_receive: Decimal = payments
                .iter()
                .filter_map(|p| if p.to == person { Some(p.value) } else { None })
                .sum();

            match person {
                Person::Named { .. } => println!("\n{}:", person.identifier()),
                Person::Unnamed { size } if size == 1 => {
                    println!("\nPessoa restante:")
                }
                Person::Unnamed { .. } => {
                    println!("\nCada uma das {} que restaram:", person.identifier())
                }
            }
            println!("    total a pagar: {total_debt:.2}");
            println!("    total a receber: {total_to_receive:.2}");

            if !debts.is_empty() {
                println!()
            }
            for p in debts {
                let value = p.value / Decimal::from(person.size());
                println!("    pagar: {value:.2} -> {}", p.to.identifier());
            }
        }
    }
}

impl FromIterator<Person> for Payments {
    fn from_iter<T: IntoIterator<Item = Person>>(iter: T) -> Self {
        let persons: Vec<Person> = iter.into_iter().collect();
        let mut payments = Vec::new();

        let num_persons: Decimal = persons.iter().map(|p| Decimal::from(p.size())).sum();

        for creditor in persons.iter() {
            if creditor.money_spent().is_zero() {
                continue;
            }

            let amount_for_each = creditor.money_spent() / num_persons;
            for debitor in persons.iter().filter(|p| p != &creditor) {
                let amount = amount_for_each * Decimal::from(debitor.size());

                payments.push(Payment::new(debitor, creditor, amount.round_dp(3)));
            }
        }

        Payments::new(&payments)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_minimize_payments() {
        // italo=222.48 maria=14.5 ana_clara=22.48 luis=146.6 guilherme=48.76 rafael=232 -p 11
        let persons = vec![
            Person::named("italo", dec!(222.48)),
            Person::named("maria", dec!(14.5)),
            Person::named("ana_clara", dec!(22.48)),
            Person::named("luis", dec!(146.6)),
            Person::named("guilherme", dec!(48.76)),
            Person::named("rafael", dec!(232)),
            Person::unnamed(5),
        ];

        let mut initial_payments: Payments = persons.clone().into_iter().collect();

        let final_payments = vec![
            Payment::new(&persons[2], &persons[3], dec!(36.22)), // ana_clara -> luis
            Payment::new(&persons[2], &persons[0], dec!(3.73)),  // ana_clara -> italo
            Payment::new(&persons[4], &persons[0], dec!(13.68)), // guilherme -> italo
            Payment::new(&persons[1], &persons[3], dec!(47.94)), // maria -> luis
            Payment::new(&persons[6], &persons[5], dec!(169.56)), // 5 pessoas -> rafael
            Payment::new(&persons[6], &persons[0], dec!(142.63)), // 5 pessoas -> italo
        ];

        initial_payments.optimize();
        let left: HashSet<Payment> = HashSet::from_iter(initial_payments.to_vec());
        let right: HashSet<Payment> = HashSet::from_iter(final_payments);

        assert_eq!(left, right);
        assert!(initial_payments.validate());
    }
}
