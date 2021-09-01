// I don't know how else to pass an Option<&mut T> as an argument multiple times, other than Option<&mut T>::as_deref_mut()
#![allow(clippy::needless_option_as_deref)]

use crate::factoring::{PollardRho, TrialDivision};
use crate::primality::{
    LucasPrimality, LucasPrimalityResult, MillerRabin, MillerRabinCompositeResult,
};

/// Observer with callbacks relating to events during the factorization of an integer
pub trait FactoringEventSubscriptor<T> {
    /// Number `n` has been factorized into parts
    /// # Arguments
    /// `n`: Number which has been factorized
    /// `primes`: Factors of `n` already known to be prime
    /// `composites`: Factors of `n` already known to be composite
    /// `unknown`: Factors of `n` of which primality is unknown so far
    fn factorized(&mut self, n: &T, primes: &[T], composites: &[T], unknown: &[T]);
    /// Factor `n` now known to be prime
    fn is_prime(&mut self, n: &T);
    /// Factor `n` now known to be composite
    fn is_composite(&mut self, n: &T);
}

/// Stub observer, for when no event callbacks are required
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EmptyFactoringEventSubscriptor {}

impl<T> FactoringEventSubscriptor<T> for EmptyFactoringEventSubscriptor {
    fn factorized(&mut self, _n: &T, _primes: &[T], _composites: &[T], _unknown: &[T]) {}
    fn is_prime(&mut self, _n: &T) {}
    fn is_composite(&mut self, _n: &T) {}
}

#[derive(Debug, Default, Clone, PartialEq)]
/// Element of the lucas certificate tree, representing one number
pub struct LucasCertificateElement<T> {
    /// The factor being certified to be prime
    pub n: T,
    /// The base for which the lucas primality test returns a _prime_ result
    pub base: T,
    /// The unique divisors of `n` - 1
    pub unique_prime_divisors: Vec<T>,
}

#[derive(Debug, Default, Clone)]
/// The certificate tree for the lucas certificate
pub struct LucasCertificate<T> {
    #[allow(missing_docs)]
    pub elements: Vec<LucasCertificateElement<T>>,
}

impl<T: Eq + Ord + Clone> LucasCertificate<T> {
    fn push(&mut self, e: LucasCertificateElement<T>) {
        match (&self.elements)
            .binary_search_by_key(&e.n, |x: &LucasCertificateElement<T>| x.n.clone())
        {
            Ok(_) => (),
            Err(i) => self.elements.insert(i, e),
        }
    }

    #[must_use]
    /// Get proof element for number `i`
    pub fn get(&self, i: &T) -> Option<&LucasCertificateElement<T>> {
        self.elements.iter().find(|x| &x.n == i)
    }

    #[must_use]
    /// Get largest proof element, presumably the element which was meant to be certified
    pub fn get_max(&self) -> Option<&LucasCertificateElement<T>> {
        self.elements.last()
    }
}

impl<T> std::convert::From<LucasCertificateElement<T>> for LucasCertificate<T> {
    fn from(x: LucasCertificateElement<T>) -> Self {
        Self { elements: vec![x] }
    }
}

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
        if self.certified_prime_check(Some(&mut certificate)) {
            return Some(certificate);
        }
        None
    }
}

impl Primality for u128 {
    fn is_prime(self) -> bool {
        use std::convert::TryFrom;
        if let Ok(x) = u64::try_from(self) {
            return x.is_prime();
        }
        self.certified_prime_check(None)
    }

    fn generate_lucas_certificate(self) -> Option<LucasCertificate<Self>> {
        let mut certificate = LucasCertificate::default();
        self.certified_prime_check(Some(&mut certificate))
            .then(|| certificate)
    }
}

impl Primality for rug::Integer {
    fn is_prime(self) -> bool {
        use std::convert::TryFrom;
        if let Ok(x) = u64::try_from(self.clone()) {
            x.is_prime()
        } else if let Ok(x) = u128::try_from(self.clone()) {
            x.is_prime()
        } else {
            self.certified_prime_check(None)
        }
    }

    fn generate_lucas_certificate(self) -> Option<LucasCertificate<Self>> {
        let mut certificate = LucasCertificate::default();
        self.certified_prime_check(Some(&mut certificate))
            .then(|| certificate)
    }
}

/// Factorize number while possible updating a lucas certificate
pub trait CertifiedFactorization: Sized {
    /// Same as [`Factoring::factor_events`], but optionally a certificate can be passed, which will be filled to certify the primality of all factors found
    fn certified_factor<T>(
        self,
        certificate: Option<&mut LucasCertificate<Self>>,
        events: T,
    ) -> Vec<Self>
    where
        T: FactoringEventSubscriptor<Self>;

        /// Given a certificate, equivalent to [`Primality::generate_lucas_certificate`].
        /// 
        /// ```
        /// use facto::{Primality, CertifiedFactorization};
        /// let mut c = facto::LucasCertificate::default();
        /// assert!(101u64.certified_prime_check(Some(&mut c)));
        /// assert_eq!(c.get_max(), 101u64.generate_lucas_certificate().unwrap().get_max())
        /// ```
    fn certified_prime_check(self, certificate: Option<&mut LucasCertificate<Self>>) -> bool;
}

impl CertifiedFactorization for u64 {
    fn certified_factor<T: FactoringEventSubscriptor<Self>>(
        self,
        mut certificate: Option<&mut LucasCertificate<Self>>,
        mut events: T,
    ) -> Vec<Self> {
        const TRIAL_THRESHHOLD: u64 = (1 << 12) - 1;

        let (mut pre_processed, exhaustive) = self.trial_division(&TRIAL_THRESHHOLD);
        if certificate.is_some() {
            for prime_factor in &pre_processed[..pre_processed.len().saturating_sub(1)] {
                prime_factor.certified_prime_check(certificate.as_deref_mut());
            }
            if exhaustive {
                pre_processed
                    .last()
                    .unwrap()
                    .certified_prime_check(certificate.as_deref_mut());
            }
        }
        if exhaustive
            || pre_processed
                .last()
                .unwrap()
                .certified_prime_check(certificate.as_deref_mut())
        {
            return pre_processed;
        }

        let mut composite_factors = vec![pre_processed.pop().unwrap()];
        let mut prime_factors = pre_processed;
        if !prime_factors.is_empty() {
            events.factorized(&self, &prime_factors, &composite_factors, &[]);
        }

        let mut pollard_rho_increment = 1;
        while let Some(current_factor) = composite_factors.last().copied() {
            match current_factor.pollard_rho(&2, &pollard_rho_increment) {
                Some(f) => {
                    composite_factors.pop();
                    let other_factor = current_factor / f;
                    events.factorized(&current_factor, &[], &[], &[f, other_factor]);
                    if f.certified_prime_check(certificate.as_deref_mut()) {
                        events.is_prime(&f);
                        prime_factors.push(f);
                    } else {
                        // FIXME: coreutils/factor uses `pollard_rho_increment + 1` to check this factor
                        // Maybe we should do too
                        events.is_composite(&f);
                        composite_factors.push(f);
                    }
                    if other_factor.certified_prime_check(certificate.as_deref_mut()) {
                        events.is_prime(&other_factor);
                        prime_factors.push(other_factor);
                    } else {
                        events.is_composite(&other_factor);
                        composite_factors.push(other_factor);
                    }
                }
                None => {
                    pollard_rho_increment += 1;
                }
            }
        }
        prime_factors.sort_unstable();
        prime_factors
    }

    fn certified_prime_check(self, certificate: Option<&mut LucasCertificate<Self>>) -> bool {
        let certificate = if let Some(certificate) = certificate {
            certificate
        } else {
            return self.is_prime();
        };
        if certificate.get(&self).is_some() {
            return true;
        }
        if self == 2 {
            if certificate.get(&self).is_none() {
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

        let mut factors =
            (self - 1).certified_factor(Some(certificate), EmptyFactoringEventSubscriptor {});
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
        mut certificate: Option<&mut LucasCertificate<Self>>,
        mut events: T,
    ) -> Vec<Self> {
        const TRIAL_THRESHHOLD: u128 = (1 << 12) - 1;

        let (mut pre_processed, exhaustive) = self.trial_division(&TRIAL_THRESHHOLD);
        if let Some(certificate) = certificate.as_deref_mut() {
            for prime_factor in &pre_processed[..pre_processed.len().saturating_sub(1)] {
                prime_factor.certified_prime_check(Some(certificate));
            }
            if exhaustive {
                pre_processed
                    .last()
                    .unwrap()
                    .certified_prime_check(Some(certificate));
            }
        }
        if exhaustive
            || pre_processed
                .last()
                .unwrap()
                .certified_prime_check(certificate.as_deref_mut())
        {
            return pre_processed;
        }

        let mut composite_factors = vec![pre_processed.pop().unwrap()];
        let mut prime_factors = pre_processed;
        if !prime_factors.is_empty() {
            events.factorized(&self, &prime_factors, &composite_factors, &[]);
        }

        let mut pollard_rho_increment = 1;
        while let Some(current_factor) = composite_factors.last().copied() {
            match current_factor.pollard_rho(&2, &pollard_rho_increment) {
                Some(f) => {
                    composite_factors.pop();
                    let other_factor = current_factor / f;
                    events.factorized(&current_factor, &[], &[], &[f, other_factor]);
                    if f.certified_prime_check(certificate.as_deref_mut()) {
                        events.is_prime(&f);
                        prime_factors.push(f);
                    } else {
                        // FIXME: coreutils/factor uses `pollard_rho_increment + 1` to check this factor
                        // Maybe we should do too
                        events.is_composite(&f);
                        composite_factors.push(f);
                    }
                    if other_factor.certified_prime_check(certificate.as_deref_mut()) {
                        events.is_prime(&other_factor);
                        prime_factors.push(other_factor);
                    } else {
                        events.is_composite(&other_factor);
                        composite_factors.push(other_factor);
                    }
                }
                None => {
                    pollard_rho_increment += 1;
                }
            }
        }
        prime_factors.sort_unstable();
        prime_factors
    }

    fn certified_prime_check(self, mut certificate: Option<&mut LucasCertificate<Self>>) -> bool {
        if self % 2 == 0 {
            return if self == 2 {
                if let Some(certificate) = certificate {
                    if certificate.get(&self).is_none() {
                        certificate.push(LucasCertificateElement {
                            n: self,
                            base: 1,
                            unique_prime_divisors: vec![1],
                        });
                    }
                }
                true
            } else {
                false
            };
        };

        // Try a few miller-rabin bases before we factor n - 1 for lucas primality
        for pre_base in 2u64..=20 {
            match self.miller_rabin(Self::from(pre_base)) {
                MillerRabinCompositeResult::Composite => return false,
                MillerRabinCompositeResult::MaybePrime => (),
            }
        }
        let mut n_minus_1_unique_prime_factors = (self - 1).certified_factor(
            certificate.as_deref_mut(),
            EmptyFactoringEventSubscriptor {},
        );
        n_minus_1_unique_prime_factors.dedup();
        for pre_base in 2u64..=20 {
            match self.lucas_primality_test(&n_minus_1_unique_prime_factors, pre_base.into()) {
                LucasPrimalityResult::Prime => {
                    if let Some(certificate) = certificate {
                        certificate.push(LucasCertificateElement {
                            n: self,
                            base: pre_base.into(),
                            unique_prime_divisors: n_minus_1_unique_prime_factors,
                        });
                    }
                    return true;
                }
                LucasPrimalityResult::Composite => return false,
                LucasPrimalityResult::Unknown => (),
            }
        }
        let mut base = 21;

        loop {
            match self.miller_rabin(base) {
                MillerRabinCompositeResult::Composite => return false,
                MillerRabinCompositeResult::MaybePrime => (),
            };
            match self.lucas_primality_test(&n_minus_1_unique_prime_factors, base) {
                LucasPrimalityResult::Prime => {
                    if let Some(certificate) = certificate {
                        certificate.push(LucasCertificateElement {
                            n: self,
                            base,
                            unique_prime_divisors: n_minus_1_unique_prime_factors,
                        });
                    }
                    return true;
                }
                LucasPrimalityResult::Composite => return false,
                LucasPrimalityResult::Unknown => (),
            }
            base += 1;
        }
    }
}

impl CertifiedFactorization for rug::Integer {
    fn certified_factor<T: FactoringEventSubscriptor<Self>>(
        self,
        mut certificate: Option<&mut LucasCertificate<Self>>,
        mut events: T,
    ) -> Vec<Self> {
        const TRIAL_THRESHHOLD: u128 = (1 << 12) - 1;

        let (mut pre_processed, exhaustive) = self.clone().trial_division(&TRIAL_THRESHHOLD.into());
        if let Some(certificate) = certificate.as_deref_mut() {
            for prime_factor in &pre_processed[..pre_processed.len().saturating_sub(1)] {
                prime_factor
                    .clone()
                    .certified_prime_check(Some(certificate));
            }
            if exhaustive {
                pre_processed
                    .last()
                    .unwrap()
                    .clone()
                    .certified_prime_check(Some(certificate));
            }
        }
        if exhaustive
            || pre_processed
                .last()
                .unwrap()
                .clone()
                .certified_prime_check(certificate.as_deref_mut())
        {
            return pre_processed;
        }

        let mut composite_factors = vec![pre_processed.pop().unwrap()];
        let mut prime_factors = pre_processed;

        if !prime_factors.is_empty() {
            events.factorized(&self, &prime_factors, &composite_factors, &[]);
        }

        let mut pollard_rho_increment = Self::from(1);
        while let Some(current_factor) = composite_factors.last().cloned() {
            match current_factor
                .clone()
                .pollard_rho(&Self::from(2), &pollard_rho_increment)
            {
                Some(f) => {
                    let other_factor = current_factor.clone() / &f;
                    events.factorized(
                        &current_factor,
                        &[],
                        &[],
                        &[f.clone(), other_factor.clone()],
                    );

                    composite_factors.pop();
                    if f.clone().certified_prime_check(certificate.as_deref_mut()) {
                        events.is_prime(&f);
                        prime_factors.push(f.clone());
                    } else {
                        // FIXME: coreutils/factor uses `pollard_rho_increment + 1` to check this factor
                        // Maybe we should do too
                        events.is_composite(&f);
                        composite_factors.push(f.clone());
                    }
                    if other_factor
                        .clone()
                        .certified_prime_check(certificate.as_deref_mut())
                    {
                        events.is_prime(&other_factor);
                        prime_factors.push(other_factor);
                    } else {
                        events.is_composite(&other_factor);
                        composite_factors.push(other_factor);
                    }
                }
                None => {
                    pollard_rho_increment += 1;
                }
            }
        }
        prime_factors.sort_unstable();
        prime_factors
    }

    fn certified_prime_check(self, mut certificate: Option<&mut LucasCertificate<Self>>) -> bool {
        if self.clone() % 2 == 0 {
            return if self == 2 {
                if let Some(certificate) = certificate {
                    if certificate.get(&self).is_none() {
                        certificate.push(LucasCertificateElement {
                            n: self,
                            base: 1.into(),
                            unique_prime_divisors: vec![1.into()],
                        });
                    }
                }
                true
            } else {
                false
            };
        };
        // Try a few miller-rabin bases before we factor n - 1 for lucas primality
        for pre_base in 2u64..=20 {
            match self.clone().miller_rabin(Self::from(pre_base)) {
                MillerRabinCompositeResult::Composite => return false,
                MillerRabinCompositeResult::MaybePrime => (),
            }
        }
        let mut n_minus_one_unique_prime_factors = Self::certified_factor(
            self.clone() - 1,
            certificate.as_deref_mut(),
            EmptyFactoringEventSubscriptor {},
        );
        n_minus_one_unique_prime_factors.dedup();

        for pre_base in 2u64..=20 {
            match self
                .clone()
                .lucas_primality_test(&n_minus_one_unique_prime_factors, pre_base.into())
            {
                LucasPrimalityResult::Prime => {
                    if let Some(certificate) = certificate {
                        certificate.push(LucasCertificateElement {
                            n: self,
                            base: pre_base.into(),
                            unique_prime_divisors: n_minus_one_unique_prime_factors,
                        });
                    }
                    return true;
                }
                LucasPrimalityResult::Composite => return false,
                LucasPrimalityResult::Unknown => (),
            }
        }
        let mut base = 21;

        loop {
            match self.clone().miller_rabin(base.into()) {
                MillerRabinCompositeResult::Composite => return false,
                MillerRabinCompositeResult::MaybePrime => (),
            };
            match self
                .clone()
                .lucas_primality_test(&n_minus_one_unique_prime_factors, base.into())
            {
                LucasPrimalityResult::Prime => {
                    if let Some(certificate) = certificate {
                        certificate.push(LucasCertificateElement {
                            n: self,
                            base: base.into(),
                            unique_prime_divisors: n_minus_one_unique_prime_factors,
                        });
                    }
                    return true;
                }
                LucasPrimalityResult::Composite => return false,
                LucasPrimalityResult::Unknown => (),
            }
            base += 1;
        }
    }
}
impl Factoring for u64 {
    fn factor_events<T: FactoringEventSubscriptor<Self>>(self, events: T) -> Vec<Self> {
        self.certified_factor(None, events)
    }
}

impl Factoring for u128 {
    fn factor_events<T: FactoringEventSubscriptor<Self>>(self, events: T) -> Vec<Self> {
        self.certified_factor(None, events)
    }
}

impl Factoring for rug::Integer {
    fn factor_events<T: FactoringEventSubscriptor<Self>>(self, events: T) -> Vec<Self> {
        self.certified_factor(None, events)
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
