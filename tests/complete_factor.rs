#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(unused)]
#![warn(single_use_lifetimes)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::unseparated_literal_suffix)]

use facto::{
    CertifiedFactorization, EmptyFactoringEventSubscriptor, LucasCertificate, PrimalityCertainty,
};
use rug::{rand::RandState, Complete};

fn get_rand_gen() -> RandState<'static> {
    let mut state = rug::rand::RandState::new();
    state.seed(
        &std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros()
            .into(),
    );
    state
}

fn check_certificate<T>(c: &LucasCertificate<T>)
where
    T: Into<rug::Integer> + Clone + Eq + Ord + std::fmt::Debug,
{
    for e in &c.elements {
        let e_n: rug::Integer = e.n.clone().into();
        if e_n == 2 {
            continue;
        }
        let mut mn: rug::Integer = e_n.clone() - 1;
        assert_eq!(
            e.base
                .clone()
                .into()
                .pow_mod(&(e_n.clone() - 1), &e_n)
                .expect("Exponentiation failed"),
            1
        );
        for ef in &e.unique_prime_divisors {
            let ef_rug: rug::Integer = ef.clone().into();
            assert!(ef_rug > 1);
            assert!(
                c.elements.binary_search_by(|x| x.n.cmp(ef)).is_ok(),
                "Certificate does not certify factor {}",
                ef_rug
            );
            assert_eq!(mn.clone() % &ef_rug, 0, "{} % {}", mn, ef_rug);
            while mn.clone() % &ef_rug == 0 {
                mn /= &ef_rug;
            }
            assert_ne!(
                e.base
                    .clone()
                    .into()
                    .pow_mod(&((e_n.clone() - 1) / &ef_rug), &e_n)
                    .expect("Exponentiation failed"),
                1
            );
        }
        assert_eq!(mn, 1);
    }
}

fn check_certified_factor<T>(n: &T, one: T)
where
    T: std::fmt::Debug
        + Clone
        + CertifiedFactorization
        + Default
        + Eq
        + Ord
        + std::fmt::Display
        + std::ops::MulAssign,
    rug::Integer: From<T>,
{
    dbg!(&n);

    let mut c = LucasCertificate::default();

    let v = n.clone().certified_factor(
        PrimalityCertainty::Certified(&mut c),
        EmptyFactoringEventSubscriptor {},
    );
    let mut re = one;
    for f in v {
        re *= f.clone();
        assert!(
            c.elements.binary_search_by(|x| x.n.cmp(&f)).is_ok(),
            "Factor {} of {} is not certified",
            f,
            n
        );
    }
    assert_eq!(n, &re);
    check_certificate(&c);
}

#[test]
fn random_test_u64() {
    let mut state = get_rand_gen();
    let threshold = rug::Integer::u_pow_u(2, 64).complete();

    for _ in 0..1000 {
        check_certified_factor(
            &threshold.clone().random_below(&mut state).to_u64_wrapping(),
            1,
        );
    }
}

#[test]
#[ignore]
fn random_test_u128() {
    let mut state = get_rand_gen();
    let threshold = rug::Integer::u_pow_u(2, 128).complete();

    for _ in 0..2 {
        check_certified_factor(
            &threshold
                .clone()
                .random_below(&mut state)
                .to_u128_wrapping(),
            1,
        );
    }
}

#[test]
fn random_test_u128_smooth() {
    let mut state = get_rand_gen();
    let threshold = rug::Integer::u_pow_u(2, 40).complete();

    for _ in 0..15 {
        let mut r = 1u128;
        for _ in 0..3 {
            r *= (threshold.clone().random_below(&mut state) + &threshold).to_u128_wrapping();
        }
        check_certified_factor(&r, 1);
    }
}

#[test]
#[ignore]
fn random_test_rug() {
    let mut state = get_rand_gen();
    let threshold = rug::Integer::u_pow_u(2, 140).complete();

    for _ in 0..2 {
        check_certified_factor(&threshold.clone().random_below(&mut state), 1.into());
    }
}

#[test]
fn random_test_rug_smooth() {
    let mut state = get_rand_gen();
    let threshold = rug::Integer::u_pow_u(2, 35).complete();

    for _ in 0..3 {
        let mut r = rug::Integer::from(1);
        for _ in 0..4 {
            r *= threshold.clone().random_below(&mut state) + &threshold;
        }
        check_certified_factor(&r, 1.into());
    }
}
