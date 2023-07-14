// I don't know how else to pass an Option<&mut T> as an argument multiple times, other than Option<&mut T>::as_deref_mut()
#![allow(clippy::needless_option_as_deref)]

mod event;
use std::convert::TryFrom;
use std::ops::{Add, Div};

use event::WrappingFactoringEventSubscriptor;
pub use event::{EmptyFactoringEventSubscriptor, FactoringEventSubscriptor};
mod certificate;
pub use certificate::{LucasCertificate, LucasCertificateElement, LucasCertificateTrait};

use crate::factoring::{PollardRho, TrialDivision};
use crate::primality::{
    LucasPrimality, LucasPrimalityResult, MillerRabin, MillerRabinCompositeResult,
};

use self::certificate::WrappingLucasCertificate;

/// Optimized methods of checking and certifying primality
pub trait Primality: Sized {
    #[allow(clippy::wrong_self_convention)]
    /// Check primality with absolute certainty
    fn is_prime(self) -> bool;
    /// Generate a lucas certificate, certifying the number's primality
    fn generate_lucas_certificate(self) -> Option<LucasCertificate<Self>>;
}

/// Factor number into it's prime factors
pub trait Factoring: Sized {
    /// Factor number, while being notified as soon as any factors are found using the observer "`events`"
    fn factor_events<T: FactoringEventSubscriptor<Self>>(self, events: T) -> Vec<Self>;

    /// Factor number
    ///
    /// # Example
    /// ```
    /// use facto::Factoring;
    /// assert_eq!(60u64.factor(), vec![2u64, 2, 3, 5])
    /// ```
    fn factor(self) -> Vec<Self> {
        Self::factor_events(self, EmptyFactoringEventSubscriptor {})
    }
}

impl Primality for u64 {
    fn is_prime(self) -> bool {
        // <https://en.wikipedia.org/wiki/Miller%E2%80%93Rabin_primality_test#Testing_against_small_sets_of_bases>
        // if n < 18,446,744,073,709,551,616 = 2^64, it is enough to test a = 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, and 37
        // <http://miller-rabin.appspot.com/>
        // 7	20-04-2011	at least 2^64	2, 325, 9375, 28178, 450775, 9780504, 1795265022	Jim Sinclair
        const REQ_PRIMES: [u64; 7] = [2, 325, 9375, 28178, 450_775, 9_780_504, 1_795_265_022];
        for base in REQ_PRIMES {
            if self.miller_rabin(base) == MillerRabinCompositeResult::Composite {
                return false;
            }
        }
        true
    }
    fn generate_lucas_certificate(self) -> Option<LucasCertificate<Self>> {
        let mut certificate = LucasCertificate::default();
        if self.certified_prime_check(PrimalityCertainty::Certified(&mut certificate)) {
            return Some(certificate);
        }
        None
    }
}

impl Primality for u128 {
    fn is_prime(self) -> bool {
        if let Ok(x) = u64::try_from(self) {
            return x.is_prime();
        }
        self.certified_prime_check(PrimalityCertainty::Guaranteed)
    }

    fn generate_lucas_certificate(self) -> Option<LucasCertificate<Self>> {
        let mut certificate = LucasCertificate::default();
        self.certified_prime_check(PrimalityCertainty::Certified(&mut certificate))
            .then_some(certificate)
    }
}

impl Primality for rug::Integer {
    #[allow(clippy::option_if_let_else)] // Looks clearer this way
    fn is_prime(self) -> bool {
        if let Ok(x) = u64::try_from(self.clone()) {
            x.is_prime()
        } else if let Ok(x) = u128::try_from(self.clone()) {
            x.is_prime()
        } else {
            self.certified_prime_check(PrimalityCertainty::Guaranteed)
        }
    }

    fn generate_lucas_certificate(self) -> Option<LucasCertificate<Self>> {
        let mut certificate = LucasCertificate::default();
        self.certified_prime_check(PrimalityCertainty::Certified(&mut certificate))
            .then_some(certificate)
    }
}

/// Factorize number while possible updating a lucas certificate
pub trait CertifiedFactorization: Sized {
    /// Same as [`Factoring::factor_events`], but optionally a certificate can be passed, which will be filled to certify the primality of all factors found
    /// # Example
    /// ```
    /// use facto::{CertifiedFactorization, PrimalityCertainty};
    /// let mut c = facto::LucasCertificate::default();
    /// let f = 10987081u128.certified_factor(
    ///     PrimalityCertainty::Certified(&mut c),
    ///     facto::EmptyFactoringEventSubscriptor{}
    /// );
    /// assert_eq!(f, vec![7, 107, 14669]);
    /// assert!(c.elements.binary_search_by_key(&7, |x| x.n).is_ok());
    /// assert!(c.elements.binary_search_by_key(&107, |x| x.n).is_ok());
    /// assert!(c.elements.binary_search_by_key(&14669, |x| x.n).is_ok());
    /// ```
    fn certified_factor<T>(self, certificate: PrimalityCertainty<Self>, events: T) -> Vec<Self>
    where
        T: FactoringEventSubscriptor<Self>;

    /// Given a certificate, equivalent to [`Primality::generate_lucas_certificate`].
    ///
    /// ```
    /// use facto::{Primality, CertifiedFactorization, PrimalityCertainty};
    /// let mut c = facto::LucasCertificate::default();
    /// assert!(101u64.certified_prime_check(PrimalityCertainty::Certified(&mut c)));
    /// assert_eq!(c.get_max(), 101u64.generate_lucas_certificate().unwrap().get_max())
    /// ```
    fn certified_prime_check(self, certificate: PrimalityCertainty<Self>) -> bool;
}

#[derive(Debug)]
/// Grade of certainty for primality check
pub enum PrimalityCertainty<'a, T> {
    /// Expensive lucas primality check to guarantee primality
    Guaranteed,
    /// Same as `Guaranteed`, but also generates the certificate
    Certified(&'a mut dyn LucasCertificateTrait<T>),
}

fn pollard_loop<T, E>(
    composite: T,
    one: &T,
    prime_factors: &mut Vec<T>,
    mut events: E,
    mut c: PrimalityCertainty<T>,
) where
    T: Clone + PollardRho + Div<Output = T> + CertifiedFactorization + Add<Output = T>,
    E: FactoringEventSubscriptor<T>,
{
    let mut pollard_rho_increment = one.clone();

    let two = one.clone() + one.clone();

    let mut composite_factors = vec![composite];
    while let Some(current_factor) = composite_factors.last().cloned() {
        #[allow(clippy::option_if_let_else)]
        match current_factor
            .clone()
            .pollard_rho(&two, &pollard_rho_increment)
        {
            Some(f) => {
                handle_factor(
                    &current_factor,
                    f,
                    &mut events,
                    &mut c,
                    &mut composite_factors,
                    prime_factors,
                );
            }
            None => {
                pollard_rho_increment = pollard_rho_increment + one.clone();
            }
        }
    }
}

fn handle_factor<T, E>(
    current_factor: &T,
    f: T,
    events: &mut E,
    c: &mut PrimalityCertainty<'_, T>,
    composite_factors: &mut Vec<T>,
    prime_factors: &mut Vec<T>,
) where
    T: Clone + PollardRho + Div<Output = T> + CertifiedFactorization + Add<Output = T>,
    E: FactoringEventSubscriptor<T>,
{
    composite_factors.pop();
    let other_factor = current_factor.clone() / f.clone();
    events.factorized(current_factor, &[], &[], &[f.clone(), other_factor.clone()]);

    let mut categorize_factor = |f: T| {
        if f.clone()
            .certified_prime_check(clone_primality_certainty(c))
        {
            events.is_prime(&f);
            prime_factors.push(f);
        } else {
            // FIXME: coreutils/factor uses `pollard_rho_increment + 1` to check this factor
            // Maybe we should do too
            events.is_composite(&f);
            composite_factors.push(f);
        }
    };
    categorize_factor(f);
    categorize_factor(other_factor);
}

fn clone_primality_certainty<'a, T>(x: &'a mut PrimalityCertainty<T>) -> PrimalityCertainty<'a, T> {
    match x {
        PrimalityCertainty::Guaranteed => PrimalityCertainty::Guaranteed,
        PrimalityCertainty::Certified(ref mut x) => PrimalityCertainty::Certified(*x),
    }
}

impl CertifiedFactorization for u64 {
    fn certified_factor<T: FactoringEventSubscriptor<Self>>(
        self,
        mut certificate: PrimalityCertainty<Self>,
        mut events: T,
    ) -> Vec<Self> {
        const TRIAL_THRESHHOLD: u64 = (1 << 12) - 1;

        let (mut pre_processed, exhaustive) = self.trial_division(&TRIAL_THRESHHOLD);
        if let PrimalityCertainty::Certified(ref mut x) = certificate {
            for prime_factor in &pre_processed[..pre_processed.len().saturating_sub(1)] {
                prime_factor.certified_prime_check(PrimalityCertainty::Certified(*x));
            }
            if exhaustive {
                pre_processed
                    .last()
                    .unwrap()
                    .certified_prime_check(PrimalityCertainty::Certified(*x));
            }
        }
        if exhaustive
            || pre_processed
                .last()
                .unwrap()
                .certified_prime_check(clone_primality_certainty(&mut certificate))
        {
            return pre_processed;
        }

        let composite_factor = pre_processed.pop().unwrap();
        let mut prime_factors = pre_processed;
        if !prime_factors.is_empty() {
            events.factorized(&self, &prime_factors, &[composite_factor], &[]);
        }

        pollard_loop(
            composite_factor,
            &1,
            &mut prime_factors,
            events,
            certificate,
        );

        prime_factors.sort_unstable();
        prime_factors
    }

    fn certified_prime_check(self, certificate: PrimalityCertainty<Self>) -> bool {
        let PrimalityCertainty::Certified(certificate) = certificate else {
            return self.is_prime();
        };
        if certificate.contains(&self) {
            return true;
        }
        if self == 2 {
            if !certificate.contains(&self) {
                certificate.push(LucasCertificateElement {
                    n: self,
                    base: 1,
                    unique_prime_divisors: vec![1],
                });
            }
            return true;
        };
        if !self.is_prime() {
            return false;
        }

        let mut factors = (self - 1).certified_factor(
            PrimalityCertainty::Certified(certificate),
            EmptyFactoringEventSubscriptor {},
        );
        factors.dedup();
        let factors = factors;

        let mut witness = 0;
        for base in 2.. {
            match self.lucas_primality_test(&factors, base){
                LucasPrimalityResult::Prime => {
                    witness = base;
                    break;
                },
                LucasPrimalityResult::Composite => panic!("We already checked for compositeness, we should never reach this. Miller rabin bases wrong?"),
                LucasPrimalityResult::Unknown => (),
            }
        }
        certificate.push(LucasCertificateElement {
            n: self,
            base: witness,
            unique_prime_divisors: factors,
        });
        true
    }
}

impl CertifiedFactorization for u128 {
    fn certified_factor<T: FactoringEventSubscriptor<Self>>(
        self,
        mut certificate: PrimalityCertainty<Self>,
        mut events: T,
    ) -> Vec<Self> {
        const TRIAL_THRESHHOLD: u128 = (1 << 12) - 1;

        if let Ok(x) = u64::try_from(self) {
            let mut wrapping_cert_buffer;
            let wrapping_certificate = match certificate {
                PrimalityCertainty::Guaranteed => PrimalityCertainty::Guaranteed,
                PrimalityCertainty::Certified(p) => {
                    wrapping_cert_buffer = Some(WrappingLucasCertificate::<u64, Self>::from(p));
                    PrimalityCertainty::Certified(wrapping_cert_buffer.as_mut().unwrap())
                }
            };
            let factoring_result = x.certified_factor(
                wrapping_certificate,
                WrappingFactoringEventSubscriptor::new(events),
            );
            return factoring_result.into_iter().map(Self::from).collect();
        }

        let (mut pre_processed, exhaustive) = self.trial_division(&TRIAL_THRESHHOLD);
        if let PrimalityCertainty::Certified(ref mut certificate) = certificate {
            for prime_factor in &pre_processed[..pre_processed.len().saturating_sub(1)] {
                prime_factor.certified_prime_check(PrimalityCertainty::Certified(*certificate));
            }
            if exhaustive {
                pre_processed
                    .last()
                    .unwrap()
                    .certified_prime_check(PrimalityCertainty::Certified(*certificate));
            }
        }
        if exhaustive
            || pre_processed
                .last()
                .unwrap()
                .certified_prime_check(clone_primality_certainty(&mut certificate))
        {
            return pre_processed;
        }

        let composite_factor = pre_processed.pop().unwrap();
        let mut prime_factors = pre_processed;
        if !prime_factors.is_empty() {
            events.factorized(&self, &prime_factors, &[composite_factor], &[]);
        }

        pollard_loop(
            composite_factor,
            &1,
            &mut prime_factors,
            events,
            certificate,
        );

        prime_factors.sort_unstable();
        prime_factors
    }

    fn certified_prime_check(self, mut certificate: PrimalityCertainty<Self>) -> bool {
        if let Some(x) = check_two(&self, clone_primality_certainty(&mut certificate)) {
            return x;
        }

        if let Ok(x) = u64::try_from(self) {
            let mut o;
            let w_c = match certificate {
                PrimalityCertainty::Guaranteed => PrimalityCertainty::Guaranteed,
                PrimalityCertainty::Certified(p) => {
                    o = Some(WrappingLucasCertificate::<u64, Self>::from(p));
                    PrimalityCertainty::Certified(o.as_mut().unwrap())
                }
            };
            return x.certified_prime_check(w_c);
        }

        let n_minus_one_unique_prime_factors = match delayed_lucas(
            &self,
            self - 1,
            clone_primality_certainty(&mut certificate),
            2u128..=20,
        ) {
            (true, None) => return true,
            (true, Some(x)) => x,
            (false, _) => return false,
        };

        miller_lucas_loop(
            21,
            self,
            certificate,
            &n_minus_one_unique_prime_factors,
            |x| *x += 1,
        )
    }
}

impl CertifiedFactorization for rug::Integer {
    fn certified_factor<T: FactoringEventSubscriptor<Self>>(
        self,
        mut certificate: PrimalityCertainty<Self>,
        mut events: T,
    ) -> Vec<Self> {
        const TRIAL_THRESHHOLD: u128 = (1 << 12) - 1;

        if let Some(x) = self.to_u128() {
            let mut o;
            let w_c = match certificate {
                PrimalityCertainty::Guaranteed => PrimalityCertainty::Guaranteed,
                PrimalityCertainty::Certified(p) => {
                    o = Some(WrappingLucasCertificate::<u128, Self>::from(p));
                    PrimalityCertainty::Certified(o.as_mut().unwrap())
                }
            };
            let r = x.certified_factor(w_c, WrappingFactoringEventSubscriptor::new(events));
            return r.into_iter().map(Self::from).collect();
        }

        let (mut pre_processed, exhaustive) = self.clone().trial_division(&TRIAL_THRESHHOLD.into());
        if let PrimalityCertainty::Certified(ref mut certificate) = certificate {
            for prime_factor in &pre_processed[..pre_processed.len().saturating_sub(1)] {
                prime_factor
                    .clone()
                    .certified_prime_check(PrimalityCertainty::Certified(*certificate));
            }
            if exhaustive {
                pre_processed
                    .last()
                    .unwrap()
                    .clone()
                    .certified_prime_check(PrimalityCertainty::Certified(*certificate));
            }
        }
        if exhaustive
            || pre_processed
                .last()
                .unwrap()
                .clone()
                .certified_prime_check(clone_primality_certainty(&mut certificate))
        {
            return pre_processed;
        }

        let composite_factor = pre_processed.pop().unwrap();
        let mut prime_factors = pre_processed;

        if !prime_factors.is_empty() {
            events.factorized(&self, &prime_factors, &[composite_factor.clone()], &[]);
        }

        pollard_loop(
            composite_factor,
            &1.into(),
            &mut prime_factors,
            events,
            certificate,
        );

        prime_factors.sort_unstable();
        prime_factors
    }

    fn certified_prime_check(self, mut certificate: PrimalityCertainty<Self>) -> bool {
        if let Some(x) = check_two(&self, clone_primality_certainty(&mut certificate)) {
            return x;
        }

        if let Some(x) = self.to_u128() {
            let mut o;
            let w_c = match certificate {
                PrimalityCertainty::Guaranteed => PrimalityCertainty::Guaranteed,
                PrimalityCertainty::Certified(p) => {
                    o = Some(WrappingLucasCertificate::<u128, Self>::from(p));
                    PrimalityCertainty::Certified(o.as_mut().unwrap())
                }
            };
            return x.certified_prime_check(w_c);
        }

        let n_minus_one_unique_prime_factors = match delayed_lucas(
            &self,
            self.clone() - 1,
            clone_primality_certainty(&mut certificate),
            (2..=20).map(Self::from),
        ) {
            (true, None) => return true,
            (true, Some(x)) => x,
            (false, _) => return false,
        };

        miller_lucas_loop(
            21.into(),
            self,
            certificate,
            &n_minus_one_unique_prime_factors,
            |x| *x += 1,
        )
    }
}

fn check_two<T>(n: &T, certificate: PrimalityCertainty<T>) -> Option<bool>
where
    T: std::ops::Rem
        + std::ops::Add
        + num_traits::One
        + num_traits::Zero
        + Clone
        + std::cmp::PartialEq,
    <T as std::ops::Rem>::Output: std::cmp::PartialEq<T>,
{
    let two = T::one() + T::one();
    if (n.clone() % two.clone()) != T::zero() {
        return None;
    }
    if n != &two {
        return Some(false);
    }
    if let PrimalityCertainty::Certified(certificate) = certificate {
        if !certificate.contains(n) {
            certificate.push(LucasCertificateElement {
                n: n.clone(),
                base: T::one(),
                unique_prime_divisors: vec![T::one()],
            });
        }
    }
    Some(true)
}

fn miller_lucas_loop<T, IncFn>(
    mut start_base: T,
    n: T,
    mut c: PrimalityCertainty<T>,
    factors: &[T],
    increment_fn: IncFn,
) -> bool
where
    T: Clone + MillerRabin + LucasPrimality,
    IncFn: Fn(&mut T),
{
    loop {
        match n.clone().miller_rabin(start_base.clone()) {
            MillerRabinCompositeResult::Composite => return false,
            MillerRabinCompositeResult::MaybePrime => (),
        };
        match n.clone().lucas_primality_test(factors, start_base.clone()) {
            LucasPrimalityResult::Prime => {
                if let PrimalityCertainty::Certified(ref mut certificate) = c {
                    certificate.push(LucasCertificateElement {
                        n,
                        base: start_base,
                        unique_prime_divisors: factors.to_vec(),
                    });
                }
                return true;
            }
            LucasPrimalityResult::Composite => return false,
            LucasPrimalityResult::Unknown => (),
        }
        increment_fn(&mut start_base);
    }
}

fn delayed_lucas<T, R>(
    n: &T,
    n_minus_1: T,
    mut c: PrimalityCertainty<T>,
    range: R,
) -> (bool, Option<Vec<T>>)
where
    T: Clone + CertifiedFactorization + MillerRabin + LucasPrimality + PartialEq,
    R: std::iter::Iterator<Item = T> + Clone,
{
    // Try a few miller-rabin bases before we factor n - 1 for lucas primality
    for pre_base in range.clone() {
        match n.clone().miller_rabin(pre_base) {
            MillerRabinCompositeResult::Composite => return (false, None),
            MillerRabinCompositeResult::MaybePrime => (),
        }
    }
    let mut n_minus_one_unique_prime_factors = T::certified_factor(
        n_minus_1,
        clone_primality_certainty(&mut c),
        EmptyFactoringEventSubscriptor {},
    );
    n_minus_one_unique_prime_factors.dedup();

    for pre_base in range {
        match n
            .clone()
            .lucas_primality_test(&n_minus_one_unique_prime_factors, pre_base.clone())
        {
            LucasPrimalityResult::Prime => {
                if let PrimalityCertainty::Certified(certificate) = c {
                    certificate.push(LucasCertificateElement {
                        n: n.clone(),
                        base: pre_base,
                        unique_prime_divisors: n_minus_one_unique_prime_factors,
                    });
                }
                return (true, None);
            }
            LucasPrimalityResult::Composite => return (false, None),
            LucasPrimalityResult::Unknown => (),
        }
    }
    (true, Some(n_minus_one_unique_prime_factors))
}

impl Factoring for u64 {
    fn factor_events<T: FactoringEventSubscriptor<Self>>(self, events: T) -> Vec<Self> {
        self.certified_factor(PrimalityCertainty::Guaranteed, events)
    }
}

impl Factoring for u128 {
    fn factor_events<T: FactoringEventSubscriptor<Self>>(self, events: T) -> Vec<Self> {
        self.certified_factor(PrimalityCertainty::Guaranteed, events)
    }
}

impl Factoring for rug::Integer {
    fn factor_events<T: FactoringEventSubscriptor<Self>>(self, events: T) -> Vec<Self> {
        self.certified_factor(PrimalityCertainty::Guaranteed, events)
    }
}

#[test]
fn bla() {
    dbg!(101u128.generate_lucas_certificate());
}

#[cfg(test)]
mod tests {
    use super::Primality;
    #[test]
    fn primality() {
        assert!(407_521_u64.is_prime());
        assert!(2u128.is_prime());
        assert!(7u128.is_prime());
    }
}
