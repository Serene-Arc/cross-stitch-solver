use crate::grid::GridCell;
use prime_factorization::Factorization;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter};

#[derive(Debug, Clone, Default)]
pub struct SymbolicSum {
    constant: usize,
    square_root_terms: HashMap<usize, usize>,
}

impl fmt::Display for SymbolicSum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.constant)?;

        let mut keys: Vec<&usize> = self.square_root_terms.keys().collect();
        keys.sort();
        for &k in &keys {
            if let Some(&coefficient) = self.square_root_terms.get(k) {
                if coefficient > 1 {
                    write!(f, " + {}√{}", coefficient, k)?;
                } else {
                    write!(f, " + √{}", k)?;
                }
            }
        }
        Ok(())
    }
}

impl SymbolicSum {
    pub fn add_distance(&mut self, first: GridCell, second: GridCell) {
        let squared_distance = first.euclidean_distance_squared(&second);
        let mut decomp_irrationals = SymbolicSum::decompose(squared_distance);
        self.constant += decomp_irrationals.remove(&1).unwrap_or(0);
        for (key, value) in decomp_irrationals {
            *self.square_root_terms.entry(key).or_insert(0) += value;
        }
    }

    fn decompose(squared_number: usize) -> HashMap<usize, usize> {
        let factors = Factorization::<u64>::run(squared_number as u64);

        // Count the exponent of each prime factor.
        let mut factor_counts = HashMap::new();
        for factor in factors.factors {
            *factor_counts.entry(factor as usize).or_insert(0) += 1;
        }

        // Calculate largest square divisor and remainder factor.
        let (largest_square_divisor, remainder_factor) =
            factor_counts
                .iter()
                .fold((1, 1), |(square, rem), (&prime, &exp)| {
                    let square_power = exp / 2;
                    let remainder = exp % 2;
                    (
                        square * prime.pow(square_power as u32),
                        rem * if remainder > 0 { prime } else { 1 },
                    )
                });

        // Construct decomposition map
        let mut decomposition = HashMap::new();
        decomposition.insert(
            if remainder_factor == 1 {
                1
            } else {
                remainder_factor
            },
            largest_square_divisor,
        );

        decomposition
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_no_constant_2_square() {
        let irrationals = SymbolicSum::decompose(4);
        assert_eq!(irrationals, HashMap::from([(1, 2)]));
    }

    #[test]
    fn test_root_no_constant_5_square() {
        let irrationals = SymbolicSum::decompose(25);
        assert_eq!(irrationals, HashMap::from([(1, 5)]));
    }

    #[test]
    fn test_root_no_constant_2_root() {
        let irrationals = SymbolicSum::decompose(2);
        assert_eq!(irrationals, HashMap::from([(2, 1)]));
    }

    #[test]
    fn test_root_no_constant_2_2_root() {
        let irrationals = SymbolicSum::decompose(8);
        assert_eq!(irrationals, HashMap::from([(2, 2)]));
    }

    #[test]
    fn test_root_no_constant_4_2_root() {
        let irrationals = SymbolicSum::decompose(32);
        assert_eq!(irrationals, HashMap::from([(2, 4)]));
    }

    #[test]
    fn test_root_no_constant_5_root() {
        let irrationals = SymbolicSum::decompose(5);
        assert_eq!(irrationals, HashMap::from([(5, 1)]));
    }

    #[test]
    fn test_symbolic_sum_string_empty() {
        let sum = SymbolicSum::default();
        assert_eq!(sum.to_string(), "0");
    }

    #[test]
    fn test_symbolic_sum_string_constant() {
        let mut sum = SymbolicSum::default();
        sum.constant = 10;
        assert_eq!(sum.to_string(), "10");
    }

    #[test]
    fn test_symbolic_sum_string_single_irrational() {
        let mut sum = SymbolicSum::default();
        sum.square_root_terms.insert(2, 1);
        assert_eq!(sum.to_string(), "0 + √2");
    }

    #[test]
    fn test_symbolic_sum_from_cells_constant() {
        let mut sum = SymbolicSum::default();
        sum.add_distance(GridCell { x: 0, y: 0 }, GridCell { x: 0, y: 1 });
        assert_eq!(sum.to_string(), "1");
    }

    #[test]
    fn test_symbolic_sum_from_cells_irrational_1() {
        let mut sum = SymbolicSum::default();
        sum.add_distance(GridCell { x: 1, y: 1 }, GridCell { x: 2, y: 0 });
        assert_eq!(sum.to_string(), "0 + √2");
    }
}
