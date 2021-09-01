#![allow(clippy::module_name_repetitions)]

mod lucas_primality;
mod miller_rabin;
pub use lucas_primality::LucasPrimality;
pub use lucas_primality::LucasPrimalityResult;
pub use miller_rabin::MillerRabin;
pub use miller_rabin::Result as MillerRabinCompositeResult;
