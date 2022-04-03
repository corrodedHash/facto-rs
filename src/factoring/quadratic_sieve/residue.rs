use redc::{element::Element, Redc};

fn eulers_criterion(n: u128, p: u128) -> bool {
    if n == 0 || n == 1 || p <= 2 {
        return true;
    }

    debug_assert_eq!(
        p % 2,
        1,
        "{p} is divisible by two, needs to be a prime number"
    );

    let f = p.setup_field();
    let wrapped_n = f.wrap_element(n);
    wrapped_n.pow(f.raw_element((p - 1) / 2)).to_normal() == 1
}

pub fn is_prime_mod_res(n: u128, prime: u128) -> bool {
    eulers_criterion(n, prime)
}

pub fn is_prime_power_mod_res(n: u128, primebase: u128, exponent: u32) -> bool {
    if n <= 1 || exponent == 0 || primebase <= 1 {
        return true;
    }
    // Handle powers of 2 differently
    if primebase == 2 {
        return match exponent {
            1 => true,
            2 => return n % 4 <= 1,
            _ => {
                // n mod 2**k is quadratic residue, if n is of the form (4**k) * (8*a + 1)
                let n = n >> (2 * (n.trailing_zeros() / 2));
                (n - 1) % 8 == 0
            }
        };
    }
    eulers_criterion(n, primebase)
}

/// Calculates r such that r*r = n mod p
/// p needs to be a prime number
///
/// [Wikipedia - Tonelli Shanks](https://en.wikipedia.org/wiki/Tonelli%E2%80%93Shanks_algorithm)
#[allow(clippy::many_single_char_names)]
pub fn tonelli_shanks(square: u128, prime_modulus: u128) -> u128 {
    if prime_modulus <= 1 {
        return 0;
    }
    if prime_modulus == 2 {
        return square % 2;
    }

    let field = prime_modulus.setup_field();
    let square_wrapped = field.wrap_element(square);

    if prime_modulus % 4 == 3 {
        return square_wrapped
            .pow(field.raw_element(prime_modulus / 4 + 1))
            .to_normal();
    }

    let s = (prime_modulus - 1).trailing_zeros();
    let q = (prime_modulus - 1) >> s;
    let non_quad_res = (2..prime_modulus)
        .find(|x| !eulers_criterion(*x, prime_modulus))
        .unwrap();
    let non_quad_wrapped = field.wrap_element(non_quad_res);

    let mut c = non_quad_wrapped.pow(field.raw_element(q));
    let mut t = square_wrapped.pow(field.raw_element(q));
    let mut r = square_wrapped.pow(field.raw_element((q / 2) + 1));
    let mut m = s;

    let one = field.wrap_element(1);

    while t.internal() != &0 && t.internal() != one.internal() {
        let mut temp_t = t;
        let mut new_m = 0;
        for i in 1..m {
            temp_t = temp_t.pow(field.raw_element(2));
            if temp_t.internal() == one.internal() {
                new_m = i;
                break;
            }
        }
        assert!(
            new_m < m,
            "{square} is a non-quadratic residue of {prime_modulus}"
        );
        let b = c.pow(field.raw_element(1 << (m - new_m - 1)));
        let b_squared = b.pow(field.raw_element(2));

        m = new_m;
        c = b_squared;
        t = t * b_squared;
        r = r * b;
    }

    if t.internal() == one.internal() {
        r.to_normal()
    } else {
        0
    }
}

pub fn prime_mod_sqrt(square: u128, prime: u128) -> u128 {
    tonelli_shanks(square, prime)
}

pub mod modulo_square_root {
    use redc::{element::Element, Redc};

    use super::prime_mod_sqrt;

    pub struct BinaryModulo {
        square: u128,
        current_power: u128,
        last_roots: Vec<u128>,
    }

    impl BinaryModulo {
        pub const fn new(square: u128) -> Self {
            Self {
                square,
                current_power: 1,
                last_roots: vec![],
            }
        }
    }

    impl std::iter::Iterator for BinaryModulo {
        type Item = Vec<u128>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.current_power == 1 {
                self.current_power = 2;
                self.last_roots = vec![self.square % 2];
                return Some(self.last_roots.clone());
            } else if self.current_power == 2 {
                self.current_power = 4;
                let bit = if self.square % 4 == 0 { 0 } else { 1 };
                self.last_roots = vec![bit, 2u128 + bit];
                return Some(self.last_roots.clone());
            }

            self.current_power *= 2;
            let round_square = self.square % self.current_power;

            self.last_roots = self
                .last_roots
                .clone()
                .into_iter()
                .flat_map(|x| {
                    [
                        x,
                        x.wrapping_add(self.current_power / 2) % self.current_power,
                    ]
                })
                .filter(|x| x.wrapping_mul(*x) % self.current_power == round_square)
                .collect();

            self.last_roots.sort_unstable();
            self.last_roots.dedup();
            Some(self.last_roots.clone())
        }
    }

    pub struct OddPrime {
        square: u128,
        modulo: u128,
        current_power: u128,
        last_root: u128,
    }

    impl OddPrime {
        pub const fn new(square: u128, modulo: u128) -> Self {
            Self {
                square,
                modulo,
                current_power: 1,
                last_root: 0,
            }
        }
    }

    impl std::iter::Iterator for OddPrime {
        type Item = u128;

        fn next(&mut self) -> Option<Self::Item> {
            if self.current_power == 1 {
                self.current_power = self.modulo;
                self.last_root = prime_mod_sqrt(self.square, self.modulo);
                return Some(self.last_root);
            }

            self.current_power = self.current_power.checked_mul(self.modulo)?;

            let field = self.current_power.setup_field();

            // Taylor series
            // f(r_k + tp^k) = f(r_k) + (-tp^k) * f'(r_k) + (-t^2p^{2k}) * f''(..) mod p^{k+1}
            // p^{2k} = 0 mod p^{k+1}
            // f(r_k) - tp^k * f'(r_k) = 0

            // r_{k+1} = r_k + t * p^k = r_k - f(r_k) / f'(r_k)
            let r_k = field.wrap_element(self.last_root);

            // f(r_k)
            let f_r_k = r_k * r_k - field.wrap_element(self.square);

            // f'(r_k)
            let f_prime_r_k = r_k * field.wrap_element(2);

            // f(r_k) / f'(r_k)
            let f_div_f_prime = f_prime_r_k.invert() * f_r_k;

            self.last_root = (r_k - f_div_f_prime).to_normal();
            Some(self.last_root)
        }
    }
}

/// For an odd prime, calculate b ** 2 = square mod prime ** k with Hensel lifting
pub fn odd_prime_power_mod_sqrt(square: u128, prime: u128, exponent: u32) -> u128 {
    if exponent == 0 {
        return 0;
    }

    modulo_square_root::OddPrime::new(square, prime)
        .nth(exponent as usize - 1)
        .unwrap()
}

pub fn binary_power_mod_sqrt(square: u128, exponent: u32) -> Vec<u128> {
    debug_assert!(is_prime_power_mod_res(square, 2, exponent as u32));
    if exponent == 0 {
        return vec![0];
    }

    modulo_square_root::BinaryModulo::new(square)
        .nth(exponent as usize - 1)
        .unwrap()
}

mod residue_test {
    #[test]
    #[ignore]
    fn all_binary_roots_print() {
        for modulus in [1 << 16] {
            println!("{modulus}");
            let mut map = std::collections::BTreeMap::<u128, Vec<u128>>::new();
            for (square, root) in (0..modulus).map(|x| (x * x % modulus, x)) {
                map.entry(square).or_default().push(root);
            }
            for (square, roots) in map.iter() {
                let a: String = roots
                    .iter()
                    .map(|x| format!("{x:>5}"))
                    .reduce(|a, b| format!("{a}, {b}"))
                    .unwrap();
                println!("{square:>5}: {a}");
            }
            println!();
        }
    }
}

#[cfg(test)]
mod test {
    use super::{
        binary_power_mod_sqrt, eulers_criterion, is_prime_power_mod_res, odd_prime_power_mod_sqrt,
        tonelli_shanks,
    };

    #[test]
    fn test_prime_power() {
        for i in [0, 1, 4, 7] {
            assert!(is_prime_power_mod_res(i, 3, 2), "{i} should be res of 9");
        }
        for i in [2, 3, 5, 6, 8] {
            assert!(
                !is_prime_power_mod_res(i, 3, 2),
                "{i} should not be res of 9"
            );
        }

        let root = odd_prime_power_mod_sqrt(7, 9 * 9 * 9, 9);
        assert_eq!(7, (root * root) % (9 * 9 * 9));
    }

    #[test]
    fn all_binary_roots() {
        for modulus_exp in 0..17 {
            let modulus = 1 << modulus_exp;
            let mut total_root_count = 0u128;
            for i in 0..modulus {
                if is_prime_power_mod_res(i, 2, modulus.trailing_zeros()) {
                    for root in binary_power_mod_sqrt(i, modulus.trailing_zeros()) {
                        assert_eq!(
                            root.wrapping_mul(root) % modulus,
                            i,
                            "{root}**2 != {i} mod {modulus}"
                        );
                        total_root_count += 1;
                    }
                }
            }
            assert_eq!(total_root_count, modulus);
        }
    }

    #[test]
    fn test_quad_res() {
        let test_primes = [101u128, 7057, 6037, 7919];
        for p in test_primes {
            let mut naive_quad_res: Vec<_> = (1..p).map(|n| ((n * n) % p, n)).collect();
            naive_quad_res.sort_unstable();
            naive_quad_res.dedup();
            for n in 2..p {
                let quad_res = naive_quad_res.binary_search_by_key(&n, |x| x.0);
                let naive_truth = quad_res.is_ok();
                let tested_truth = eulers_criterion(n, p);
                assert_eq!(
                    naive_truth, tested_truth,
                    "{n} should be quadres mod {p}: {naive_truth}",
                );

                if let Ok(q) = quad_res {
                    let (square, root) = naive_quad_res[q];
                    let r = tonelli_shanks(square, p);
                    assert!(
                        r == root || p - r == root,
                        "sqrt({square}) mod {p} should be {root}, turned out to be {r} and {neg_r}",
                        neg_r = p - r
                    );
                }
            }
        }
    }
}
