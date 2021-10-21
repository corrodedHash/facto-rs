use crate::redc::{self, Redc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum JacobiResult {
    QuadraticResidue,
    NonQuadraticResidue,
    Factor,
}

fn jacobi(mut n: u64, mut k: u64) -> JacobiResult {
    k %= n;
    let mut quadratic_residue = false;
    while k != 0 {
        let zero_count = k.trailing_zeros();
        k >>= zero_count;
        if zero_count & 1 == 1 && (n & 0b111 == 3 || n & 0b111 == 5) {
            quadratic_residue = !quadratic_residue;
        }
        std::mem::swap(&mut k, &mut n);
        if n & 0b11 == 3 && k & 0b11 == 3 {
            quadratic_residue = !quadratic_residue;
        }
        k %= n;
    }
    if n == 1 {
        if quadratic_residue {
            JacobiResult::QuadraticResidue
        } else {
            JacobiResult::NonQuadraticResidue
        }
    } else {
        JacobiResult::Factor
    }
}
fn find_D(n: u64) -> u64 {
    let mut start = 5;
    let mut negative = false;
    let mut k = if negative { n - start } else { start };
    while jacobi(n, k) != JacobiResult::NonQuadraticResidue {
        start += 2;
        negative = !negative;
        k = if negative { n - start } else { start };
    }
    start
}

fn double_UV(u: u64, v: u64, q_k: u64, n: u64, field: &redc::Field<u64>) -> (u64, u64) {
    let new_u = u64::redc(field, u128::from(u) * u128::from(v));
    let v_squared = u64::redc(field, u128::from(v) * u128::from(v));
    let new_v = ((2u128 * u128::from(n - q_k) + u128::from(v_squared)) % u128::from(n)) as u64;
    (new_u, new_v)
}

fn increment_UV(u: u64, v: u64, p: u64, two_inv: u64, field: &redc::Field<u64>) -> (u64, u64) {
    let p_times_u = u64::redc(field, u128::from(p) * u128::from(u));
    let new_u = p_times_u.checked_add(v).unwrap_or_else(|| );
    todo!();
}

fn calc_UV(n: u64, q: u64, p: u64) -> u64 {
    let field = <u64 as Redc>::setup_field(n);
    let u = u64::to_montgomery_unchecked(1, &field);
    let p = u64::to_montgomery_unchecked(p, &field);
    let v = p;
    let two_inverse = u64::to_montgomery_unchecked(2, &field).mod_pow(n - 2, &field);

    todo!();
}

#[test]
fn test_jacobi() {
    let residue_n_k = vec![
        (3, vec![2, 5, 8, 11, 14, 17, 20, 23, 26, 29]),
        (5, vec![2, 3, 7, 8, 12, 13, 17, 18, 22, 23, 27, 28]),
        (7, vec![3, 5, 6, 10, 12, 13, 17, 19, 20, 24, 26, 27]),
        (11, vec![2, 6, 7, 8, 10, 13, 17, 18, 19, 21, 24, 28, 29, 30]),
        (13, vec![2, 5, 6, 7, 8, 11, 15, 18, 19, 20, 21, 24, 28]),
        (15, vec![7, 11, 13, 14, 22, 26, 28, 29]),
        (17, vec![3, 5, 6, 7, 10, 11, 12, 14, 20, 22, 23, 24]),
        (19, vec![2, 3, 8, 10, 12, 13, 14, 15, 18, 21, 22, 27, 29]),
        (21, vec![2, 8, 10, 11, 13, 19, 23, 29]),
        (23, vec![5, 7, 10, 11, 14, 15, 17, 19, 20, 21, 22, 28, 30]),
        (27, vec![2, 5, 8, 11, 14, 17, 20, 23, 26, 29]),
        (29, vec![2, 3, 8, 10, 11, 12, 14, 15, 17, 18]),
        (31, vec![3, 6, 11, 12, 13, 15, 17, 21, 22, 23]),
        (33, vec![5, 7, 10, 13, 14, 19, 20, 23, 26, 28]),
        (35, vec![2, 6, 8, 18, 19, 22, 23, 24, 26]),
        (37, vec![2, 5, 6, 8, 13, 14, 15, 17, 18, 19, 20, 22, 23, 24]),
        (39, vec![7, 14, 17, 19, 23, 28, 29]),
        (41, vec![3, 6, 7, 11, 12, 13, 14, 15, 17, 19, 22, 24, 26]),
        (43, vec![2, 3, 5, 7, 8, 12, 18, 19, 20, 22, 26, 27, 28, 29]),
        (45, vec![2, 7, 8, 13, 17, 22, 23, 28]),
        (47, vec![5, 10, 11, 13, 15, 19, 20, 22, 23, 26, 29, 30]),
        (51, vec![2, 7, 8, 10, 22, 26, 28]),
        (53, vec![2, 3, 5, 8, 12, 14, 18, 19, 20, 21, 22, 23, 26, 27]),
        (55, vec![3, 6, 12, 19, 21, 23, 24, 27, 29]),
        (57, vec![5, 10, 11, 13, 17, 20, 22, 23, 26]),
        (59, vec![2, 6, 8, 10, 11, 13, 14, 18, 23, 24, 30]),
    ];
    let non_residue_n_k = vec![
        (3, vec![1, 4, 7, 10, 13, 16, 19, 22, 25, 28]),
        (5, vec![1, 4, 6, 9, 11, 14, 16, 19, 21, 24, 26, 29]),
        (7, vec![1, 2, 4, 8, 9, 11, 15, 16, 18, 22, 23, 25, 29, 30]),
        (9, vec![1, 2, 4, 5, 7, 8, 10, 11, 13, 14, 16, 17, 19, 20]),
        (11, vec![1, 3, 4, 5, 9, 12, 14, 15, 16, 20, 23, 25, 26, 27]),
        (13, vec![1, 3, 4, 9, 10, 12, 14, 16, 17, 22, 23, 25, 27, 29]),
        (15, vec![1, 2, 4, 8, 16, 17, 19, 23]),
        (17, vec![1, 2, 4, 8, 9, 13, 15, 16, 18, 19, 21, 25, 26, 30]),
        (19, vec![1, 4, 5, 6, 7, 9, 11, 16, 17, 20, 23, 24, 25]),
        (21, vec![1, 4, 5, 16, 17, 20, 22, 25, 26]),
        (23, vec![1, 2, 3, 4, 6, 8, 9, 12, 13, 16, 18, 24, 25]),
        (25, vec![1, 2, 3, 4, 6, 7, 8, 9, 11, 12, 13, 14, 16, 17]),
        (27, vec![1, 4, 7, 10, 13, 16, 19, 22, 25, 28]),
        (29, vec![1, 4, 5, 6, 7, 9, 13, 16, 20, 22]),
        (31, vec![1, 2, 4, 5, 7, 8, 9, 10, 14, 16, 18, 19]),
        (33, vec![1, 2, 4, 8, 16, 17, 25, 29]),
        (35, vec![1, 3, 4, 9, 11, 12, 13, 16, 17, 27, 29]),
        (37, vec![1, 3, 4, 7, 9, 10, 11, 12, 16, 21, 25, 26, 27]),
        (39, vec![1, 2, 4, 5, 8, 10, 11, 16, 20, 22, 25]),
        (41, vec![1, 2, 4, 5, 8, 9, 10, 16, 18, 20, 21, 23, 25]),
        (43, vec![1, 4, 6, 9, 10, 11, 13, 14, 15, 16, 17, 21]),
        (45, vec![1, 4, 11, 14, 16, 19, 26, 29]),
        (47, vec![1, 2, 3, 4, 6, 7, 8, 9, 12, 14, 16, 17, 18]),
        (49, vec![1, 2, 3, 4, 5, 6, 8, 9, 10, 11, 12, 13, 15, 16]),
        (51, vec![1, 4, 5, 11, 13, 14, 16, 19, 20, 23, 25, 29]),
        (53, vec![1, 4, 6, 7, 9, 10, 11, 13, 15, 16, 17, 24, 25]),
        (55, vec![1, 2, 4, 7, 8, 9, 13, 14, 16, 17, 18, 26, 28]),
        (57, vec![1, 2, 4, 7, 8, 14, 16, 25, 28, 29]),
        (59, vec![1, 3, 4, 5, 7, 9, 12, 15, 16, 17, 19, 20]),
    ];
    for (n, ks) in residue_n_k {
        for k in ks {
            assert_eq!(jacobi(n, k), JacobiResult::QuadraticResidue, "{}/{}", n, k);
        }
    }
    for (n, ks) in non_residue_n_k {
        for k in ks {
            assert_eq!(
                jacobi(n, k),
                JacobiResult::NonQuadraticResidue,
                "{}/{}",
                n,
                k
            );
        }
    }
    for n in 1..20 {
        let n = 2 * n + 1;
        for k in 1..20 {
            let k = n * k;
            assert_eq!(jacobi(n, k), JacobiResult::Factor, "{}/{}", n, k);
        }
    }
}

pub trait LucasStrongPossiblePrime {
    fn lucas_strong_possible_prime(self) -> bool;
}

impl LucasStrongPossiblePrime for u64 {
    fn lucas_strong_possible_prime(self) -> bool {
        let D = find_D(self);
        let Q = self - D + 1;
        let P = 1;

        todo!();
    }
}
