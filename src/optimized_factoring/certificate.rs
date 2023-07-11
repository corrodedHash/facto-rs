use std::{convert::TryFrom, marker::PhantomData};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
/// Element of the lucas certificate tree, representing one number
pub struct LucasCertificateElement<T> {
    /// The factor being certified to be prime
    pub n: T,
    /// The base for which the lucas primality test returns a _prime_ result
    pub base: T,
    /// The unique divisors of `n` - 1
    pub unique_prime_divisors: Vec<T>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Default, Clone)]
/// The certificate tree for the lucas certificate
pub struct LucasCertificate<T> {
    #[allow(missing_docs)]
    pub elements: Vec<LucasCertificateElement<T>>,
}

/// Trait enabling adding and querying parts of the certificate
#[allow(clippy::module_name_repetitions)]
pub trait LucasCertificateTrait<T>: std::fmt::Debug {
    /// Push new element to the certificate chain
    fn push(&mut self, e: LucasCertificateElement<T>);

    /// Check if element `i` is part of the certificate
    fn contains(&self, i: &T) -> bool;
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct WrappingLucasCertificate<'a, OutputType, WrappedType> {
    wrapped: &'a mut dyn LucasCertificateTrait<WrappedType>,
    _phantom_from: PhantomData<OutputType>,
}

fn change_element<OutputType, WrappedType: From<OutputType>>(
    c: LucasCertificateElement<OutputType>,
) -> LucasCertificateElement<WrappedType> {
    LucasCertificateElement {
        n: c.n.into(),
        base: c.base.into(),
        unique_prime_divisors: c
            .unique_prime_divisors
            .into_iter()
            .map(std::convert::Into::into)
            .collect(),
    }
}

impl<'a, FromType, ToType> From<&'a mut dyn LucasCertificateTrait<FromType>>
    for WrappingLucasCertificate<'a, ToType, FromType>
where
    FromType: Ord + Clone + std::fmt::Debug,
{
    fn from(x: &'a mut dyn LucasCertificateTrait<FromType>) -> Self {
        WrappingLucasCertificate {
            wrapped: x,
            _phantom_from: PhantomData,
        }
    }
}

impl<F, T> LucasCertificateTrait<F> for WrappingLucasCertificate<'_, F, T>
where
    F: Clone + TryFrom<T> + std::fmt::Debug,
    T: From<F> + Clone + std::fmt::Debug,
{
    fn push(&mut self, e: LucasCertificateElement<F>) {
        self.wrapped.push(change_element::<F, T>(e));
    }

    fn contains(&self, i: &F) -> bool {
        self.wrapped.contains(&i.clone().into())
    }
}

impl<T> LucasCertificate<T>
where
    T: Eq + Clone,
{
    #[must_use]
    /// Get proof element for number `i`
    pub fn get(&self, i: &T) -> Option<LucasCertificateElement<T>> {
        self.elements.iter().find(|x| &x.n == i).cloned()
    }

    #[must_use]
    /// Get largest proof element, presumably the element which was meant to be certified
    pub fn get_max(&self) -> Option<LucasCertificateElement<T>> {
        self.elements.last().cloned()
    }
}

impl<T: Eq + Ord + Clone + std::fmt::Debug> LucasCertificateTrait<T> for LucasCertificate<T> {
    fn push(&mut self, e: LucasCertificateElement<T>) {
        match self
            .elements
            .binary_search_by_key(&e.n, |x: &LucasCertificateElement<T>| x.n.clone())
        {
            Ok(_) => (),
            Err(i) => self.elements.insert(i, e),
        }
    }

    fn contains(&self, i: &T) -> bool {
        self.elements.binary_search_by_key(&i, |x| &x.n).is_ok()
    }
}

impl<T> std::convert::From<LucasCertificateElement<T>> for LucasCertificate<T> {
    fn from(x: LucasCertificateElement<T>) -> Self {
        Self { elements: vec![x] }
    }
}
