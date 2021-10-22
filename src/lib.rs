//! Integer factorization, primality checking and primality certification
//!
//! ```
//! use facto::{Factoring, Primality};
//! assert_eq!(65u64.factor(), vec![5, 13]);
//! assert!(13u64.is_prime());
//! assert!(!14u64.is_prime());
//! dbg!(101u64.generate_lucas_certificate());
//! ```

#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(unused)]
#![warn(single_use_lifetimes)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::unseparated_literal_suffix)]

/// Factorization algorithms for integers
pub mod factoring;
mod optimized_factoring;
/// Primality checking algorithms for integers
pub mod primality;
mod util;
/// Montgomery multiplication methods
pub use redc;

pub use optimized_factoring::{
    CertifiedFactorization, EmptyFactoringEventSubscriptor, Factoring, FactoringEventSubscriptor,
    LucasCertificate, LucasCertificateElement, Primality, PrimalityCertainty,
};

#[doc(no_inline)]
pub use rug::Integer;

#[cfg(test)]
mod tests {
    use crate::optimized_factoring::Factoring;

    #[test]
    fn test_factor() {
        assert_eq!(
            (4_294_967_279u64 * 4_294_967_291).factor(),
            &[4_294_967_279, 4_294_967_291]
        );
    }
}
