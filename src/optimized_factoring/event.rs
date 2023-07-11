use std::marker::PhantomData;

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

pub struct WrappingFactoringEventSubscriptor<Inner, F, To>
where
    Inner: FactoringEventSubscriptor<To>,
    To: From<F>,
{
    inner: Inner,
    _phantom_from: PhantomData<F>,
    _phantom_to: PhantomData<To>,
}

impl<Inner, F, To> FactoringEventSubscriptor<F> for WrappingFactoringEventSubscriptor<Inner, F, To>
where
    Inner: FactoringEventSubscriptor<To>,
    To: From<F>,
    F: Clone,
{
    fn factorized(&mut self, n: &F, primes: &[F], composites: &[F], unknown: &[F]) {
        let p_wrapped: Vec<To> = primes.iter().map(|x| To::from(x.clone())).collect();
        let c_wrapped: Vec<To> = composites.iter().map(|x| To::from(x.clone())).collect();
        let u_wrapped: Vec<To> = unknown.iter().map(|x| To::from(x.clone())).collect();
        self.inner
            .factorized(&To::from(n.clone()), &p_wrapped, &c_wrapped, &u_wrapped);
    }

    fn is_prime(&mut self, n: &F) {
        self.inner.is_prime(&To::from(n.clone()));
    }

    fn is_composite(&mut self, n: &F) {
        self.inner.is_composite(&To::from(n.clone()));
    }
}

impl<Inner, F, To> WrappingFactoringEventSubscriptor<Inner, F, To>
where
    Inner: FactoringEventSubscriptor<To>,
    To: From<F>,
    F: Clone,
{
    pub const fn new(inner: Inner) -> Self {
        Self {
            inner,
            _phantom_to: std::marker::PhantomData,
            _phantom_from: std::marker::PhantomData,
        }
    }
}
