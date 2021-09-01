use redc::Redc;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Result of the lucas primality test [`LucasPrimality`]
pub enum LucasPrimalityResult {
    /// Number is guaranteed prime
    Prime,
    /// Number is guaranteed composite
    Composite,
    /// Test was indecisive
    Unknown,
}

#[allow(clippy::module_name_repetitions)]
/// Test number for primality using the lucas primality test
///
/// <https://en.wikipedia.org/wiki/Lucas_primality_test>
pub trait LucasPrimality: Sized {
    /// Check `self` for primality
    /// # Arguments
    /// * `self`: Number to be checked for primality
    /// * `n_minus_1_unique_prime_factors`: Prime factorization of `self` - 1
    /// * `base`: Base used to test `self`. Even if `self` is prime, not all bases are going to return [`LucasPrimalityResult`]::Prime, multiple bases may need to be tested
    fn lucas_primality_test(
        self,
        n_minus_1_unique_prime_factors: &[Self],
        base: Self,
    ) -> LucasPrimalityResult;
}

impl LucasPrimality for u64 {
    fn lucas_primality_test(
        self,
        n_minus_1_unique_prime_factors: &[Self],
        base: Self,
    ) -> LucasPrimalityResult {
        let field = self.setup_field();
        let base = base.to_montgomery_unchecked(&field);
        let one = 1u64.to_montgomery_unchecked(&field);
        let n_minus_one = self - 1;
        if base.mod_pow(n_minus_one, &field) != one {
            return LucasPrimalityResult::Composite;
        }
        for factor in n_minus_1_unique_prime_factors {
            if base.mod_pow(n_minus_one / *factor, &field) == one {
                return LucasPrimalityResult::Unknown;
            }
        }
        LucasPrimalityResult::Prime
    }
}

impl LucasPrimality for u128 {
    fn lucas_primality_test(
        self,
        n_minus_1_unique_prime_factors: &[Self],
        base: Self,
    ) -> LucasPrimalityResult {
        let field = self.setup_field();
        let base = base.to_montgomery_unchecked(&field);
        let one = 1u128.to_montgomery_unchecked(&field);
        let n_minus_one = self - 1;
        if base.mod_pow(n_minus_one, &field) != one {
            return LucasPrimalityResult::Composite;
        }
        for factor in n_minus_1_unique_prime_factors {
            if base.mod_pow(n_minus_one / *factor, &field) == one {
                return LucasPrimalityResult::Unknown;
            }
        }
        LucasPrimalityResult::Prime
    }
}

impl LucasPrimality for rug::Integer {
    fn lucas_primality_test(
        self,
        n_minus_1_unique_prime_factors: &[Self],
        base: Self,
    ) -> LucasPrimalityResult {
        let field = self.clone().setup_field();
        let base = base.to_montgomery_unchecked(&field);
        let one = Self::from(1).to_montgomery_unchecked(&field);
        let n_minus_one: Self = self - 1;
        if base.clone().mod_pow(n_minus_one.clone(), &field) != one {
            return LucasPrimalityResult::Composite;
        }
        for factor in n_minus_1_unique_prime_factors {
            if base.clone().mod_pow(n_minus_one.clone() / factor, &field) == one {
                return LucasPrimalityResult::Unknown;
            }
        }
        LucasPrimalityResult::Prime
    }
}

#[cfg(test)]
mod tests {
    use super::{LucasPrimality, LucasPrimalityResult};
    /// # Panics
    /// Should never happen, happens when primality can not be proved or disproved
    pub fn exhaustive_lucas_primality_test<T>(
        n: &T,
        n_minus_1_unique_prime_factors: &[T],
    ) -> Option<T>
    where
        T: LucasPrimality + num_traits::PrimInt + num_traits::NumAssignOps,
    {
        let mut base = T::one() + T::one();
        while &base < n {
            match n.lucas_primality_test(n_minus_1_unique_prime_factors, base) {
                LucasPrimalityResult::Prime => return Some(base),
                LucasPrimalityResult::Composite => return None,
                LucasPrimalityResult::Unknown => {}
            }
            base += T::one();
        }
        panic!()
    }

    #[test]
    fn test_lucas() {
        // https://en.wikipedia.org/wiki/Lucas_primality_test#Example
        assert_eq!(
            71u64.lucas_primality_test(&[2, 5, 7], 17),
            LucasPrimalityResult::Unknown
        );
        assert_eq!(
            71u64.lucas_primality_test(&[2, 5, 7], 11),
            LucasPrimalityResult::Prime
        );
        assert!(exhaustive_lucas_primality_test(&442_069_u64, &[2, 3, 11, 17, 197]).is_some());
        assert!(exhaustive_lucas_primality_test(
            &782_689_174_619_698_081_u64,
            &[2, 3, 5, 7, 53, 122_347, 11_974_561]
        )
        .is_some());
        assert!(exhaustive_lucas_primality_test(&442_069_u128, &[2, 3, 11, 17, 197]).is_some());
        assert!(exhaustive_lucas_primality_test(
            &782_689_174_619_698_081_u128,
            &[2, 3, 5, 7, 53, 122_347, 11_974_561]
        )
        .is_some());
    }
}
