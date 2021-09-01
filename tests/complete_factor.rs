use facto::{CertifiedFactorization, EmptyFactoringEventSubscriptor, LucasCertificate};

fn check_certified_factor_u64(n: u64) {
    dbg!(n);

    let mut c = LucasCertificate::default();

    let v = n.certified_factor(Some(&mut c), EmptyFactoringEventSubscriptor {});
    let mut re = 1u64;
    for f in v {
        re *= f;
        assert!(
            c.get(&f).is_some(),
            "Factor {} of {} is not certified",
            f,
            n
        );
    }
    assert_eq!(n, re);
    for e in &c.elements {
        if e.n == 2 {
            continue;
        }
        let mut mn = e.n - 1;
        assert_eq!(
            rug::Integer::from(e.base)
                .pow_mod(&(e.n - 1).into(), &e.n.into())
                .expect("Exponentiation failed"),
            rug::Integer::from(1)
        );
        for ef in &e.unique_prime_divisors {
            assert!(ef > &1);
            assert!(
                c.get(ef).is_some(),
                "Certificate does not certify factor {}",
                ef
            );
            assert_eq!(mn % ef, 0, "{} % {}", mn, ef);
            while mn % ef == 0 {
                mn /= ef;
            }
            assert_ne!(
                rug::Integer::from(e.base)
                    .pow_mod(&((e.n - 1) / ef).into(), &e.n.into())
                    .expect("Exponentiation failed"),
                rug::Integer::from(1)
            );
        }
        assert_eq!(mn, 1);
    }
}

#[test]
fn random_test_u64() {
    let seed = rug::Integer::from(
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    );

    let mut u64_state: rug::Integer = seed | 1;

    for _ in 0..1000 {
        u64_state = &u64_state * &u64_state + rug::Integer::from(1);
        u64_state %= rug::Integer::from(rug::Integer::u_pow_u(2, 64));
        check_certified_factor_u64(u64_state.to_u64_wrapping());
    }
}

fn check_certified_factor_u128(n: u128) {
    dbg!(n);
    let mut c = LucasCertificate::default();
    let v = n.certified_factor(Some(&mut c), EmptyFactoringEventSubscriptor {});
    let mut re = 1u128;
    for f in v {
        re *= f;
        assert!(
            c.get(&f).is_some(),
            "Factor {} of {} is not certified",
            f,
            n
        );
    }
    assert_eq!(n, re);
    for e in &c.elements {
        if e.n == 2 {
            continue;
        }
        let mut mn = e.n - 1;
        assert_eq!(
            rug::Integer::from(e.base)
                .pow_mod(&(e.n - 1).into(), &e.n.into())
                .expect("Exponentiation failed"),
            rug::Integer::from(1)
        );
        for ef in &e.unique_prime_divisors {
            assert!(ef > &1);
            assert!(
                c.get(ef).is_some(),
                "Certificate does not certify factor {}",
                ef
            );
            assert_eq!(mn % ef, 0, "{} % {}", mn, ef);
            while mn % ef == 0 {
                mn /= ef;
            }
            assert_ne!(
                rug::Integer::from(e.base)
                    .pow_mod(&((e.n - 1) / ef).into(), &e.n.into())
                    .expect("Exponentiation failed"),
                rug::Integer::from(1)
            );
        }
        assert_eq!(mn, 1);
    }
}
#[test]
#[ignore]
fn random_test_u128() {
    let seed = rug::Integer::from(
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    );

    let mut u128_state: rug::Integer = seed | 1;

    for _ in 0..5 {
        u128_state = &u128_state * &u128_state + rug::Integer::from(1);
        u128_state %= rug::Integer::from(rug::Integer::u_pow_u(2, 128));
        check_certified_factor_u128(u128_state.to_u128_wrapping());
    }
}

fn check_certified_factor_rug(n: rug::Integer) {
    dbg!(&n);
    let mut c = LucasCertificate::default();
    let v = n
        .clone()
        .certified_factor(Some(&mut c), EmptyFactoringEventSubscriptor {});
    let mut re = rug::Integer::from(1);
    for f in v {
        re *= &f;
        assert!(
            c.get(&f).is_some(),
            "Factor {} of {} is not certified",
            f,
            n
        );
    }
    assert_eq!(n, re);
    for e in &c.elements {
        if e.n == 2 {
            continue;
        }
        let mut mn: rug::Integer = e.n.clone() - 1;
        assert_eq!(
            e.base
                .clone()
                .pow_mod(&(e.n.clone() - 1), &e.n)
                .expect("Exponentiation failed"),
            rug::Integer::from(1)
        );
        for ef in &e.unique_prime_divisors {
            assert!(ef > &1);
            assert!(
                c.get(ef).is_some(),
                "Certificate does not certify factor {}",
                ef
            );
            assert_eq!(mn.clone() % ef, 0, "{} % {}", mn, ef);
            while mn.clone() % ef == 0 {
                mn /= ef;
            }
            assert_ne!(
                e.base
                    .clone()
                    .pow_mod(&((e.n.clone() - 1) / ef), &e.n)
                    .expect("Exponentiation failed"),
                rug::Integer::from(1)
            );
        }
        assert_eq!(mn, 1);
    }
}

#[test]
#[ignore]
fn random_test_rug() {
    let mut seed = rug::Integer::from(
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    );
    for _ in 0..5 {
        seed *= seed.clone();
        seed += 1;
        seed %= rug::Integer::from(rug::Integer::u_pow_u(2, 140));
    }

    for _ in 0..2 {
        seed = seed.clone() * &seed + rug::Integer::from(1);
        seed %= rug::Integer::from(rug::Integer::u_pow_u(2, 140));
        check_certified_factor_rug(seed.clone());
    }
}
