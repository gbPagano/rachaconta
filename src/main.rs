use std::collections::HashMap;

use clap::Parser;

fn parse_key_val(s: &str) -> Result<(String, f64), String> {
    let (key, value) = s.split_once('=').ok_or_else(|| {
        format!(
            "argumento inválido, esperado formato 'chave=valor': '{}'",
            s
        )
    })?;

    let parsed_value = value.parse::<f64>().map_err(|e| {
        format!(
            "erro ao processar o valor '{}' da chave '{}': {}",
            value, key, e
        )
    })?;

    Ok((key.to_string(), parsed_value))
}

/// Define os argumentos da linha de comando
#[derive(Parser, Debug)]
#[command(
    version,
    author,
    about = "Exemplo de CLI que aceita --num e pares chave=valor"
)]
struct Args {
    /// O número a ser fornecido
    #[arg(short, long)]
    num: usize,

    /// Lista de pares chave=valor posicionais
    #[arg(
        required = true,
        value_parser = parse_key_val
    )]
    pairs: Vec<(String, f64)>, // <-- Mude de volta para Vec<(String, String)>
}

#[derive(Default, Debug)]
struct Debts {
    amount_to_receive_for_each: f64,
    payments: Vec<(String, f64)>,
}

fn main() {
    let args = Args::parse();

    if args.pairs.len() > args.num {
        eprintln!(
            "Erro: a conta nao fecha! (Número de pares: {}, --num: {})",
            args.pairs.len(),
            args.num
        );
        std::process::exit(1);
    }

    let num = args.num as f64;

    let pairs_map: HashMap<String, f64> = args.pairs.into_iter().collect();

    let mut debts_map: HashMap<String, Debts> = HashMap::new();

    let mut total_debt = 0.;

    for (creditor, value) in &pairs_map {
        total_debt += value;
        let amount_to_receive = value / num;
        let c_debt = debts_map.entry(creditor.to_string()).or_default();
        c_debt.amount_to_receive_for_each += amount_to_receive;

        for debtor in pairs_map.keys() {
            if debtor == creditor {
                continue;
            }
            let p_debt = debts_map.entry(debtor.to_string()).or_default();
            p_debt.payments.push((creditor.clone(), amount_to_receive));
        }
    }

    println!("Valor total da conta: {total_debt:.2}");
    println!("    {:.2} para cada", total_debt / num);

    for (person, debt) in &debts_map {
        let mut total_to_receive = debt.amount_to_receive_for_each * (num - debts_map.len() as f64);
        let mut total_to_pay = 0.;

        println!("\n{person} deve:");
        for (creditor, val) in &debt.payments {
            let payment = val - debt.amount_to_receive_for_each;
            if payment < 0. {
                total_to_receive += -payment;
                continue;
            }
            total_to_pay += payment;
            if payment >= 0.01 {
                println!("    pagar: {payment:.2} -> {creditor}");
            }
        }

        println!("\n    total a pagar: {total_to_pay:.2}");
        println!("    total a receber: {total_to_receive:.2}");
    }

    println!("\nAs outras {} pessoas devem:", args.num - debts_map.len());
    let mut total_to_pay = 0.;
    for (creditor, debt) in &debts_map {
        let payment = debt.amount_to_receive_for_each;
        total_to_pay += payment;
        println!("    pagar: {payment:.2} -> {creditor}");
    }
    println!("\n    total a pagar: {total_to_pay:.2}");
}
