use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, Sub};

/// Representa um valor monetário em centavos.
///
/// `Money` armazena valores monetários como inteiros (1 décimo de centavos) para evitar
/// problemas de precisão de ponto flutuante em cálculos financeiros.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Money(usize);

impl Money {
    pub fn raw(&self) -> usize {
        self.0
    }

    /// Retorna o valor como um número decimal (em reais/dólares/etc.).
    pub fn decimal(&self) -> f64 {
        self.0 as f64 / 1000.
    }
}

impl<T> From<T> for Money
where
    T: Into<f64>,
{
    fn from(value: T) -> Self {
        let cents: f64 = (value.into() * 1000.).round();
        Self(cents as usize)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.2}", self.decimal())
    }
}

// Add
impl Add for Money {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl<T> Add<T> for Money
where
    T: Into<f64>,
{
    type Output = Money;

    fn add(self, rhs: T) -> Self::Output {
        let rhs_money = Money::from(rhs);
        Money(self.0 + rhs_money.0)
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Money(0), |acc, x| Money(acc.0 + x.0))
    }
}

// Sub
impl Sub for Money {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl<T> Sub<T> for Money
where
    T: Into<f64>,
{
    type Output = Money;
    fn sub(self, rhs: T) -> Self::Output {
        let rhs_money = Money::from(rhs);
        Money(self.0 - rhs_money.0)
    }
}

// Mul
impl<T> Mul<T> for Money
where
    T: Into<f64>,
{
    type Output = Money;
    fn mul(self, rhs: T) -> Self::Output {
        (self.decimal() * rhs.into()).into()
    }
}

// Div
impl<T> Div<T> for Money
where
    T: Into<f64>,
{
    type Output = Money;
    fn div(self, rhs: T) -> Self::Output {
        (self.decimal() / rhs.into()).into()
    }
}

impl<T> DivAssign<T> for Money
where
    T: Into<f64>,
{
    fn div_assign(&mut self, rhs: T) {
        *self = Money(self.0 + (self.decimal() / rhs.into()) as usize);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_money_from_numbers() {
        assert_eq!(Money::from(20), Money(20000));
        assert_eq!(Money::from(139.94), Money(139940));
    }

    #[test]
    fn test_from_rounds_correctly() {
        assert_eq!(Money::from(10.5556), Money(10556));
        assert_eq!(Money::from(10.5554), Money(10555));
    }

    #[test]
    fn test_read_money() {
        let m = Money::from(20);
        assert_eq!(m.0, 20000);
        assert_eq!(m.decimal(), 20.0);

        let m = Money::from(19.952);
        assert_eq!(m.0, 19952);
        assert_eq!(m.decimal(), 19.952);
    }

    #[test]
    fn test_add_money() {
        assert_eq!(Money::from(30) + 20, Money::from(50));
        assert_eq!(Money::from(30) + Money::from(19.9), Money::from(49.9));
    }

    #[test]
    fn test_sub() {
        assert_eq!(Money::from(30) - 20, Money::from(10));
        assert_eq!(Money::from(30) - Money::from(19.9), Money::from(10.1));
    }

    #[test]
    fn test_mul_money() {
        assert_eq!(Money::from(30) * 2, Money::from(60));
        assert_eq!(Money::from(30) * 1.5, Money::from(45));
    }

    #[test]
    fn test_div_money() {
        assert_eq!(Money::from(30) / 2, Money::from(15));
        assert_eq!(Money::from(30) / 1.5, Money::from(20));
    }
}
