#![allow(dead_code)]

//! [Paper on self-initializing quadratic sieve](https://citeseerx.ist.psu.edu/viewdoc/summary?doi=10.1.1.26.6924)

use crate::{factoring::PollardRho, util::NumUtil};
use redc::{element::Element, Redc};

mod bitvector;
use bitvector::BitVector;
mod residue;
use residue::tonelli_shanks;
pub trait QuadraticSieve {
    fn quadratic_sieve(self) -> Self;
}

#[derive(Default, Debug)]
struct PrimeIterator {
    last_prime: u128,
}
impl Iterator for PrimeIterator {
    type Item = u128;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.last_prime = match self.last_prime {
                0 => 2,
                1 => panic!("Huh?"),
                2 => 3,
                _ => self.last_prime + 2,
            };
            if crate::Primality::is_prime(self.last_prime) {
                return Some(self.last_prime);
            }
        }
    }
}

type XYElement<'a> = (redc::element::PrimIntElement<'a, u128>, rug::Integer);

fn linear_combination(
    n: u128,
    matrix: &mut Vec<BitVector>,
    xy: &mut Vec<XYElement>,
    prime_count: usize,
) -> u128 {
    for i in 0..prime_count {
        let mut hunter_index = None;
        for index in 0..matrix.len() {
            if matrix[index].trailing_zeros() == i {
                if let Some(src_index) = hunter_index {
                    let (left, right) = {
                        let (l, r) = matrix.split_at_mut(index);
                        (&mut l[src_index], &mut r[0])
                    };
                    right.add(left);

                    let (left_xy, right_xy): (&mut XYElement, &mut XYElement) = {
                        let (l, r) = xy.split_at_mut(index);
                        (&mut l[src_index], &mut r[0])
                    };
                    right_xy.0 = right_xy.0 * left_xy.0;
                    right_xy.1 *= &left_xy.1;

                    if right.is_zero() {
                        let y = right_xy.1.clone().sqrt();
                        debug_assert_eq!(y.clone().square(), right_xy.1);
                        let y = (y % n).to_u128().unwrap();
                        let x = right_xy.0.to_normal();
                        let diff = if x > y { x - y } else { y - x };
                        let g = u128::gcd(diff, n);
                        if g != n && g != 1 {
                            return g;
                        }
                    }
                } else {
                    hunter_index = Some(index);
                }
            }
        }
    }
    0
}

fn get_log_approximations(sieve_size: usize, n: u128, primes: &[u32]) -> (Vec<u8>, u128) {
    let mut log_approximation = vec![0u8; sieve_size];
    let ceil_sq = n.integer_square_root() + 1;
    let try_count_log_underapproximation = usize::MAX.trailing_ones() - sieve_size.trailing_zeros();
    for p in primes.iter().copied().map(u128::from) {
        let p_log_overapproximation = u128::BITS + 1 - p.leading_zeros();
        let max_power =
            try_count_log_underapproximation.saturating_sub(2) / p_log_overapproximation;
        for p_power in (0..=std::cmp::min(5, max_power)).scan(1, |x, _| {
            *x *= p;
            Some(*x)
        }) {
            let n_sqrt_mod_p = tonelli_shanks(n % p_power, p_power);
            let neg_sqrt_mod_p = p_power - n_sqrt_mod_p;
            let neg_ceil_sq_mod_p = p_power - (ceil_sq % p_power);

            // (x + ceil(sqrt(n))) ** 2 - n = 0 mod p
            // => (x + ceil(sqrt(n))) ** 2 = n mod p
            // => x = sqrt(n) - ceil(sqrt(n)) mod p
            let x_neg = (neg_ceil_sq_mod_p + neg_sqrt_mod_p) % p_power;
            let x_pos = (neg_ceil_sq_mod_p + n_sqrt_mod_p) % p_power;
            let start_x = if x_neg == x_pos {
                [x_neg, sieve_size as u128]
            } else {
                [x_neg, x_pos]
            };
            for start in start_x {
                for i in (start as usize..log_approximation.len()).step_by(p_power as usize) {
                    log_approximation[i] += u8::try_from(p_log_overapproximation).unwrap();
                }
            }
        }
    }
    (log_approximation, ceil_sq)
}

fn gather_relations(
    n: u128,
    sieve_size: usize,
    primes: &[u32],
) -> Vec<(u128, rug::Integer, BitVector)> {
    let (log_approximation, ceil_sq) = get_log_approximations(sieve_size, n, primes);

    let mut last_log_approx = 0u8;
    let mut result = vec![];
    for (i, content) in log_approximation.iter().enumerate() {
        if content < &last_log_approx {
            continue;
        }
        let y = (rug::Integer::from(i) + ceil_sq).square() - n;
        last_log_approx = u8::try_from(y.significant_bits()).unwrap();
        if content < &last_log_approx {
            continue;
        }
        let mut factor_vector = BitVector::new(primes.len());

        let mut pollard_rho_increment: rug::Integer = 1u32.into();
        let mut composites = vec![y.clone()];
        while !composites.is_empty() {
            let x = composites.last().unwrap().clone();
            if let Some(factor) = x.clone().pollard_rho(&2u32.into(), &pollard_rho_increment) {
                let other_factor = x.clone() / &factor;
                if let Some(f) = factor.to_u32() {
                    match primes.binary_search(&f) {
                        Ok(f_index) => factor_vector.flip(f_index),
                        Err(_) => composites.push(factor.clone()),
                    }
                } else {
                    composites.push(factor);
                }
                if let Some(other_f) = other_factor.to_u32() {
                    match primes.binary_search(&other_f) {
                        Ok(f_index) => factor_vector.flip(f_index),
                        Err(_) => composites.push(other_factor.clone()),
                    }
                } else {
                    composites.push(other_factor);
                }
            }
            pollard_rho_increment += 1u32;
        }
        result.push((i as u128, y, factor_vector));
    }
    result
}

fn data_collection(n: u128) -> u128 {
    assert_eq!(n % 2, 1);
    let quad_res_primes: Vec<_> = PrimeIterator::default()
        .filter(|x| residue::is_prime_mod_res(n, *x))
        .take(10_000)
        .map(|x| x as u32)
        .collect();

    let relations = gather_relations(n, 100_000, &quad_res_primes);
    let field = n.setup_field();
    let (mut xy, mut matrix) = {
        relations
            .into_iter()
            .map(|(x, y, vector)| ((field.wrap_element(x), y), vector))
            .unzip()
    };
    dbg!(linear_combination(
        n,
        &mut matrix,
        &mut xy,
        quad_res_primes.len()
    ))
}

#[test]
fn bla() {
    data_collection(15347);
    // data_collection(85_070_591_730_234_614_113_402_964_855_534_653_469);
}

impl QuadraticSieve for u128 {
    fn quadratic_sieve(self) -> Self {
        todo!()
    }
}
