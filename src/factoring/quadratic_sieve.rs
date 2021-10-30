#![allow(dead_code)]
use crate::util::NumUtil;
use redc::{element::Element, Redc};

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

fn eulers_criterion(n: u128, p: u128) -> bool {
    if n == 0 || n == 1 {
        return true;
    }
    if p == 2 {
        return true;
    }
    debug_assert_eq!(p % 2, 1);

    let f = p.setup_field();
    let wrapped_n = f.wrap_element(n);
    wrapped_n.pow(f.raw_element((p - 1) / 2)).to_normal() == 1
}

/// Calculates r such that r*r = n mod p
/// p needs to be a prime number
///
/// <https://en.wikipedia.org/wiki/Tonelli%E2%80%93Shanks_algorithm>
#[allow(clippy::many_single_char_names)]
fn tonelli_shanks(square: u128, prime_modulus: u128) -> u128 {
    if prime_modulus == 2 {
        return square % prime_modulus;
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
    let mut non_quad_res = 2;
    while eulers_criterion(non_quad_res, prime_modulus) {
        non_quad_res += 1;
    }
    let non_quad_res = non_quad_res;
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
            "{} is a non-quadratic residue of {}",
            square,
            prime_modulus
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

#[test]
fn test_quad_res() {
    let test_primes = [101u128, 7057, 6037, 7919];
    for p in test_primes {
        let mut naive_quad_res = vec![];
        for n in 1..p {
            naive_quad_res.push(((n * n) % p, n));
        }
        naive_quad_res.sort_unstable();
        naive_quad_res.dedup();
        for n in 2..p {
            let quad_res = naive_quad_res.binary_search_by_key(&n, |x| x.0);
            let naive_truth = quad_res.is_ok();
            let tested_truth = eulers_criterion(n, p);
            assert_eq!(
                naive_truth, tested_truth,
                "{} should be quadres mod {}: {}",
                n, p, naive_truth
            );
            if let Ok(q) = quad_res {
                let (square, root) = naive_quad_res[q];
                let r = tonelli_shanks(square, p);
                assert!(
                    r == root || p - r == root,
                    "sqrt({}) mod {} should be {}, turned out to be {} and {}",
                    square,
                    p,
                    root,
                    r,
                    p - r,
                );
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BitVector {
    elements: Vec<u128>,
}
impl std::fmt::Debug for BitVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BitVector {{\n\t")?;
        for x in &self.elements {
            write!(f, "{:032X}", x)?;
        }
        write!(f, "\n}}")?;
        Ok(())
    }
}

impl BitVector {
    fn new(size: usize) -> Self {
        Self {
            elements: vec![0; (size + 127) / 128],
        }
    }
    fn trailing_zeros(&self) -> usize {
        let mut r = 0usize;
        for e in &self.elements {
            if 0u128.trailing_zeros() != e.trailing_zeros() {
                return r + e.trailing_zeros() as usize;
            }
            r += 0u128.trailing_zeros() as usize;
        }
        r
    }
    fn is_zero(&self) -> bool {
        self.elements.iter().all(|x| x == &0)
    }
    fn add(&mut self, other: &Self) {
        for (s, o) in self.elements.iter_mut().zip(other.elements.iter()) {
            *s ^= o;
        }
    }
    const fn bit_helper(index: usize) -> (u128, usize) {
        let bit_index = index % 128;
        let cell_index = index / 128;
        let mask = 1u128 << bit_index;
        (mask, cell_index)
    }
    fn set(&mut self, index: usize, value: bool) {
        let (mask, cell_index) = Self::bit_helper(index);
        let cell = self
            .elements
            .get_mut(cell_index)
            .expect("Index out of bounds");
        if value {
            *cell |= mask;
        } else {
            *cell &= !mask;
        }
    }
    fn get(&self, index: usize) -> bool {
        let (mask, cell_index) = Self::bit_helper(index);
        let cell = self.elements.get(cell_index).expect("Index out of bounds");
        (*cell & mask) != 0
    }
    fn flip(&mut self, index: usize) {
        let (mask, cell_index) = Self::bit_helper(index);
        let cell = self
            .elements
            .get_mut(cell_index)
            .expect("Index out of bounds");
        *cell ^= mask;
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

fn data_collection(n: u128) -> u128 {
    assert_eq!(n % 2, 1);
    let quad_res_primes: Vec<_> = PrimeIterator::default()
        .filter(|x| eulers_criterion(n, *x))
        .take(10_000)
        .collect();

    let sq = n.integer_square_root() + 1;
    let sieve_size = 100_000;
    let mut a = vec![];
    a.reserve(sieve_size);
    for x in 0u128..(sieve_size as u128) {
        a.push((x + sq) * (x + sq) - n);
    }

    let mut exponent_matrix = vec![BitVector::new(sieve_size); quad_res_primes.len()];

    for (matrix_index, p) in (0..).zip(&quad_res_primes) {
        let p = *p;
        let x_neg = (p + p - tonelli_shanks(n % p, p) - (sq % p)) % p;
        let x_pos = (p + tonelli_shanks(n % p, p) - (sq % p)) % p;
        let start_x = if p == 2 {
            [x_neg, a.len() as u128]
        } else {
            [x_neg, x_pos]
        };
        for start_x in start_x {
            for i in (start_x as usize..a.len()).step_by(p as usize) {
                let mut odd_count = false;
                while a[i] % p == 0 {
                    a[i] /= p;
                    odd_count = !odd_count;
                }
                if odd_count {
                    exponent_matrix[matrix_index].flip(i);
                }
            }
        }
    }
    let hit_count = a.iter().filter(|x| x == &&1).count();
    let mut matrix = vec![BitVector::new(quad_res_primes.len()); hit_count];
    let field = n.setup_field();
    let mut xy = vec![];
    xy.reserve(hit_count);

    let relevant_row_ids = a
        .iter()
        .zip(0usize..)
        .filter(|(x, _)| x == &&1)
        .map(|(_, id)| id);

    for (new_index, old_index) in (0usize..).zip(relevant_row_ids) {
        let new_row = &mut matrix[new_index];
        for (row_index, old_row) in (0usize..).zip(&exponent_matrix) {
            new_row.set(row_index, old_row.get(old_index));
        }
        xy.push((
            field.wrap_element(sq + old_index as u128),
            rug::Integer::from(old_index as u128 + sq).square() - n,
        ));
    }
    for (x,y) in &xy{
        println!("{} {}", x.to_normal(), y);
    }
    dbg!(hit_count);
    dbg!(linear_combination(
        n,
        &mut matrix,
        &mut xy,
        quad_res_primes.len()
    ))
}

#[test]
fn bla() {
    // data_collection(15347);
    data_collection(85_070_591_730_234_614_113_402_964_855_534_653_469);
}

impl QuadraticSieve for u128 {
    fn quadratic_sieve(self) -> Self {
        todo!()
    }
}
