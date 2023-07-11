use crate::util::NumUtil;
use num_traits::PrimInt;

/// Find prime factors using naive trial division
pub trait TrialDivision: Sized + Clone {
    /// Returns factors below or equal `inclusive_bound`
    ///
    /// Last element in vector may not be prime, except when the second element of the tuple is true
    fn trial_division(self, inclusive_bound: &Self) -> (Vec<Self>, bool);
    /// Try all numbers up to the square root of `self`
    fn exhaustive_trial_division(self) -> Vec<Self> {
        self.clone().trial_division(&self).0
    }
}

fn p_trial_division<T: PrimInt + NumUtil>(mut n: T, inclusive_bound: &T) -> (Vec<T>, bool) {
    const PRE_PRIMES: [u8; 3] = [2u8, 3, 5];
    const TEST_DELTA: [u8; 2] = [1, 5];
    const ROUND_INCREMENT: u8 = 6;
    let mut result = vec![];
    for prime in PRE_PRIMES {
        let prime = T::from(prime).unwrap();
        while n % prime == T::zero() {
            result.push(prime);
            n = n / prime;
        }
    }
    let mut max_possible_factor = n.integer_square_root();
    let mut current_factor = T::from(ROUND_INCREMENT).unwrap();
    loop {
        let mut changed = false;
        for delta in TEST_DELTA {
            let f = current_factor + T::from(delta).unwrap();
            while n % f == T::zero() {
                result.push(f);
                n = n / f;
                changed = true;
            }
        }
        if n == T::one() {
            return (result, true);
        }

        if changed {
            max_possible_factor = n.integer_square_root();
        }

        if current_factor > max_possible_factor {
            result.push(n);
            return (result, true);
        }
        if &current_factor > inclusive_bound {
            result.push(n);
            return (result, false);
        }
        current_factor = current_factor + T::from(ROUND_INCREMENT).unwrap();
    }
}

macro_rules! prim_trial_division {
    ($p:ty) => {
        impl TrialDivision for $p {
            fn trial_division(self, inclusive_bound: &Self) -> (Vec<Self>, bool) {
                p_trial_division(self, inclusive_bound)
            }
        }
    };
}

prim_trial_division!(u8);
prim_trial_division!(u16);
prim_trial_division!(u32);
prim_trial_division!(u64);
prim_trial_division!(u128);

impl TrialDivision for rug::Integer {
    fn trial_division(mut self, inclusive_bound: &Self) -> (Vec<Self>, bool) {
        use rug::Assign;
        const PRE_PRIMES: [u32; 3] = [2, 3, 5];
        const TEST_DELTA: [u32; 2] = [1, 5];
        const ROUND_INCREMENT: u32 = 6;
        let mut result = vec![];
        for prime in PRE_PRIMES {
            while self.is_divisible_u(prime) {
                result.push(prime.into());
                self /= prime;
            }
        }
        let mut max_possible_factor = Self::from(self.sqrt_ref());
        let mut current_factor = Self::from(ROUND_INCREMENT);
        let mut f = Self::new();
        loop {
            let mut changed = false;
            for delta in TEST_DELTA {
                f.assign(&current_factor + delta);
                while self.is_divisible(&f) {
                    self /= &f;
                    result.push(f.clone());
                    changed = true;
                }
            }
            if self == 1 {
                return (result, true);
            }

            if changed {
                max_possible_factor.assign(self.sqrt_ref());
            }

            if current_factor > max_possible_factor {
                result.push(self);
                return (result, true);
            }
            if &current_factor > inclusive_bound {
                result.push(self);
                return (result, false);
            }
            current_factor += ROUND_INCREMENT;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::factoring::TrialDivision;

    #[test]
    fn test_trial_division() {
        assert_eq!(
            u64::MAX.trial_division(&6_700_417).0,
            &[3u64, 5, 17, 257, 641, 65537, 6_700_417]
        );
        assert_eq!(
            2_147_483_647_u32.exhaustive_trial_division(),
            &[2_147_483_647_u32]
        );
    }
}
