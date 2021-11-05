use redc::{element::Element, Redc};

fn eulers_criterion(n: u128, p: u128) -> bool {
    if n == 0 || n == 1 {
        return true;
    }
    if p == 2 {
        return true;
    }
    debug_assert_eq!(
        p % 2,
        1,
        "{} is divisible by two, needs to be a prime number",
        p
    );

    let f = p.setup_field();
    let wrapped_n = f.wrap_element(n);
    wrapped_n.pow(f.raw_element((p - 1) / 2)).to_normal() == 1
}

pub fn is_prime_mod_res(n: u128, prime: u128) -> bool {
    eulers_criterion(n, prime)
}

pub fn is_prime_power_mod_res(mut n: u128, primebase: u128, exponent: u32) -> bool {
    if n <= 1 {
        return true;
    }
    if exponent == 0 {
        return true;
    }
    // Handle powers of 2 differently
    if primebase == 2 {
        if exponent == 1 {
            return true;
        }
        if exponent == 2 {
            return n % 4 <= 1;
        }
        // n mod 2**k is quadratic residue, if n is of the form (4**k) * (8*a + 1)
        while n % 4 == 0 {
            n /= 4;
        }
        n = n.saturating_sub(1);
        return n % 8 == 0;
    }
    eulers_criterion(n, primebase)
}

/// Calculates r such that r*r = n mod p
/// p needs to be a prime number
///
/// <https://en.wikipedia.org/wiki/Tonelli%E2%80%93Shanks_algorithm>
#[allow(clippy::many_single_char_names)]
pub fn tonelli_shanks(square: u128, prime_modulus: u128) -> u128 {
    if prime_modulus == 1 {
        return 0;
    }
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

pub fn prime_mod_sqrt(square: u128, prime: u128) -> u128 {
    tonelli_shanks(square, prime)
}

pub fn binary_power_mod_sqrt(square: u128, exponent: u128) -> (u128, u128) {
    if square == 0 {
        return (0, 0);
    }
    match exponent {
        0 => return (0, 0),
        1 => return (square % 2, square % 2),
        2 => {
            if square % 4 == 0 {
                return (0, 2);
            }
            return (1, 3);
        }
        _ => (),
    }
    // while current_exponent != exponent {}
    return (0, 0);
}

/// Calculate b ** 2 = square mod prime ** k with Hensel lifting
pub fn prime_power_mod_sqrt(square: u128, primepower: u128, prime: u128) -> u128 {
    // Special case square root mod 2, because
    // the derivative of b * b - square is 2b, which is 0b mod 2, which is zero
    if prime == 2 {}
    let mut result = prime_mod_sqrt(square, prime);
    let mut current_prime_power = prime;
    while current_prime_power < primepower {
        current_prime_power *= prime;
        let field = current_prime_power.setup_field();
        let wrapped_result = field.wrap_element(result);
        let f_r_k = wrapped_result * wrapped_result - field.wrap_element(square);
        let f_prime_r_k = wrapped_result * field.wrap_element(2);
        let f_prime_r_k_inv = f_prime_r_k.invert();
        let f_div_f_prime = f_prime_r_k_inv * f_r_k;
        result = (wrapped_result - f_div_f_prime).to_normal();
    }
    result
}

#[test]
fn test_prime_power() {
    for i in [0, 1, 4, 7] {
        assert!(is_prime_power_mod_res(i, 3, 2), "{} should be res of 9", i);
    }
    for i in [2, 3, 5, 6, 8] {
        assert!(
            !is_prime_power_mod_res(i, 3, 2),
            "{} should not be res of 9",
            i
        );
    }

    let root = prime_power_mod_sqrt(7, 9 * 9 * 9, 9);
    assert_eq!(7, (root * root) % (9 * 9 * 9));
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
