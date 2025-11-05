mod money;
mod payment;
mod person;
mod solver;

use clap::Parser;

use money::Money;
use person::Person;

/// Uma CLI para dividir contas de forma justa
///
/// Calcula quanto cada pessoa deve pagar ou receber após uma série de gastos compartilhados
#[derive(Parser, Debug)]
#[command(name = "rachaconta", version)]
struct Args {
    /// Define o número total de pessoas que devem dividir os gastos.
    ///
    /// Por padrão, a conta é dividida igualmente apenas entre quem realizou
    /// algum pagamento. Use esta opção quando houver pessoas que não pagaram
    /// nada mas devem participar da divisão.
    ///
    /// Exemplo: Se 5 pessoas jantaram mas apenas 2 pagaram, use -p 5    
    #[arg(short = 'p', long = "pessoas", value_name = "NÚMERO")]
    total_persons: Option<usize>,

    /// Lista dos gastos individuais no formato NOME=VALOR
    ///
    /// Cada pagamento deve ser informado como nome da pessoa seguido de
    /// igual e o valor pago.
    ///
    /// Exemplos:
    ///   Rafael=50.00 Maria=30.50 "Ana Clara"=100
    #[arg(
        required = true,
        value_parser = parse_key_val,
        value_name = "NOME=VALOR"
    )]
    initial_payments: Vec<(String, f64)>,
}

fn main() {
    let args = Args::parse();

    let initial_payments = args.initial_payments;
    let total_persons = args.total_persons.unwrap_or(initial_payments.len());

    if initial_payments.len() > total_persons {
        eprintln!(
            "Erro: a conta não fecha! {} pessoa(s) pagaram, mas você informou apenas {} pessoa(s) no total.",
            initial_payments.len(),
            total_persons
        );
        eprintln!(
            "Dica: aumente -p para pelo menos {} ou remova a opção -p para dividir apenas entre quem pagou.",
            initial_payments.len()
        );
        std::process::exit(1);
    }

    let mut persons: Vec<_> = initial_payments
        .iter()
        .map(|p| Person::named(&p.0, p.1.into()))
        .collect();

    let remaining = total_persons - initial_payments.len();
    if remaining > 0 {
        persons.push(Person::unnamed(remaining));
    }

    let payments = solver::calc_payments(&persons);
    let payments = solver::optimize_payments(&payments);

    for person in &persons {
        let debts: Vec<_> = payments.iter().filter(|p| p.from == *person).collect();

        let mut total_debt: Money = debts.iter().map(|p| p.value).sum();
        if let Person::Unnamed { size } = person {
            total_debt = ((total_persons / size) as f64).into();
        }

        let total_to_receive: Money = payments
            .iter()
            .filter_map(|p| if p.to == *person { Some(p.value) } else { None })
            .sum();

        println!("\n{}:", person.identifier());
        println!("    total a pagar: {total_debt:.2}");
        println!("    total a receber: {total_to_receive:.2}");
        if !debts.is_empty() {
            println!()
        }

        for p in debts {
            if let Person::Unnamed { size } = person {
                println!(
                    "    pagar: {:.2} -> {}",
                    p.value / *size as f64,
                    p.to.identifier()
                );
            } else {
                println!("    pagar: {:.2} -> {}", p.value, p.to.identifier());
            }
        }
    }
}

fn parse_key_val(s: &str) -> Result<(String, f64), String> {
    let (k, v) = s.split_once('=').ok_or("use o formato NOME=VALOR")?;
    Ok((
        k.into(),
        v.parse().map_err(|_| format!("número inválido: {v}"))?,
    ))
}
