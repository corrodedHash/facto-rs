use redc::{Field, Redc};
use twoword::TwoWord;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Result of the miller rabin compositeness check
/// As a single Miller-Rabin test can only certify compositeness, a failure to do so does not guarantee the primality of the number.
pub enum Result {
    /// Number is guaranteed composite
    Composite,
    /// Number was not proven to be composite, and may be prime
    MaybePrime,
}

/// Implements the miller rabin compositeness test
///
/// <https://en.wikipedia.org/wiki/Miller%E2%80%93Rabin_primality_test>
pub trait MillerRabin: Sized {
    /// May return `MaybePrime` for composite numbers, 'strong probable primes'.
    ///
    /// When returning `Composite`, `self` is definitely a composite number
    ///
    /// # Examples
    /// ```
    /// use facto::primality::{MillerRabin, MillerRabinCompositeResult};
    /// assert_eq!(9u64.miller_rabin(2), MillerRabinCompositeResult::Composite);
    /// assert_eq!(101u128.miller_rabin(2), MillerRabinCompositeResult::MaybePrime);
    /// ```
    fn miller_rabin(self, base: Self) -> Result;
}

impl MillerRabin for u64 {
    fn miller_rabin(self, base: Self) -> Result {
        if self == 2 {
            return Result::MaybePrime;
        }
        if self % 2 == 0 {
            return Result::Composite;
        }
        let n_minus_one = self - 1;
        let s = n_minus_one.trailing_zeros();
        let d = n_minus_one >> s;

        let field = self.setup_field();
        let base = base.to_montgomery(&field);
        if base == 0 {
            return Result::MaybePrime;
        }
        let one = 1u64.to_montgomery_unchecked(&field);
        let mut base_power = base.mod_pow(d, &field);
        let neg_one_mod = n_minus_one.to_montgomery_unchecked(&field);
        if base_power == one {
            return Result::MaybePrime;
        }
        if base_power == neg_one_mod {
            return Result::MaybePrime;
        }
        for _ in 1..s {
            base_power = field.redc(u128::from(base_power) * u128::from(base_power));
            if base_power == neg_one_mod {
                return Result::MaybePrime;
            }
        }
        Result::Composite
    }
}

impl MillerRabin for u128 {
    fn miller_rabin(self, base: Self) -> Result {
        if self == 2 {
            return Result::MaybePrime;
        }
        if self % 2 == 0 {
            return Result::Composite;
        }
        let n_minus_one = self - 1;
        let s = n_minus_one.trailing_zeros();
        let d = n_minus_one >> s;

        let field = self.setup_field();
        let base = base.to_montgomery(&field);
        if base == 0 {
            return Result::MaybePrime;
        }
        let one = 1u128.to_montgomery_unchecked(&field);
        let mut base_power = base.mod_pow(d, &field);
        let neg_one_mod = n_minus_one.to_montgomery_unchecked(&field);
        if base_power == one {
            return Result::MaybePrime;
        }
        if base_power == neg_one_mod {
            return Result::MaybePrime;
        }
        for _ in 1..s {
            base_power = field.redc(TwoWord::mult(base_power, base_power));
            if base_power == neg_one_mod {
                return Result::MaybePrime;
            }
        }
        Result::Composite
    }
}

impl MillerRabin for rug::Integer {
    fn miller_rabin(self, base: Self) -> Result {
        if self == 2 {
            return Result::MaybePrime;
        }
        if self.clone() % 2 == 0 {
            return Result::Composite;
        }
        let n_minus_one: Self = self.clone() - 1;
        let s = if let Some(x) = n_minus_one.find_one(0) {
            x
        } else {
            // Must be zero, zero is composite (I guess?)
            return Result::Composite;
        };
        let d = n_minus_one.clone() >> s;

        let field = self.setup_field();
        let base = base.to_montgomery(&field);
        if base == 0 {
            return Result::MaybePrime;
        }
        let one = Self::from(1).to_montgomery_unchecked(&field);
        let mut base_power = base.mod_pow(d, &field);
        let neg_one_mod = n_minus_one.to_montgomery_unchecked(&field);
        if base_power == one {
            return Result::MaybePrime;
        }
        if base_power == neg_one_mod {
            return Result::MaybePrime;
        }
        for _ in 1..s {
            base_power = field.redc(base_power.square());
            if base_power == neg_one_mod {
                return Result::MaybePrime;
            }
        }
        Result::Composite
    }
}

#[cfg(test)]
mod tests {
    use crate::primality::MillerRabinCompositeResult;

    use super::MillerRabin;

    #[test]
    fn test_miller_rabin() {
        // https://primes.utm.edu/lists/2small/0bit.html
        // ten least k's for which 2n-k is prime.
        // const DEFINITIVE_PRIMES_DELTA: [u64; 10] =
        //     [25, 165, 259, 301, 375, 387, 391, 409, 457, 471];

        assert_eq!(
            173u64.miller_rabin(2),
            MillerRabinCompositeResult::MaybePrime
        );
        assert_eq!(
            (53u64 * 17).miller_rabin(2),
            MillerRabinCompositeResult::Composite
        );
        assert_eq!(
            173u128.miller_rabin(2),
            MillerRabinCompositeResult::MaybePrime
        );
        assert_eq!(
            (53u128 * 17).miller_rabin(2),
            MillerRabinCompositeResult::Composite
        );
    }
}
