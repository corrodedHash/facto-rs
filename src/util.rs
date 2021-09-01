use num_traits::PrimInt;

#[allow(clippy::module_name_repetitions)]
pub trait NumUtil {
    fn gcd(u: Self, v: Self) -> Self;
    fn integer_square_root(self) -> Self;
}

fn p_gcd<T>(mut u: T, mut v: T) -> T
where
    T: PrimInt,
{
    if u == T::zero() {
        return v;
    }
    if v == T::zero() {
        return u;
    }
    u = u.unsigned_shr(u.trailing_zeros());
    v = v.unsigned_shr(v.trailing_zeros());
    if u == v {
        return u;
    }
    if v > u {
        std::mem::swap(&mut u, &mut v);
    }
    let two = T::one() + T::one();
    p_gcd((u - v) / two, v)
}

fn p_integer_square_root<T>(n: T) -> T
where
    T: PrimInt,
{
    let two = T::one() + T::one();
    let mut result = n / two;
    if result == T::zero() {
        return n;
    }
    let mut next_result = (result + n / result) / two;
    while next_result < result {
        result = next_result;
        next_result = (result + n / result) / two;
    }
    result
}

macro_rules! prim_int_util {
    ($p:ty) => {
        impl NumUtil for $p {
            fn integer_square_root(self) -> Self {
                p_integer_square_root(self)
            }
            fn gcd(u: Self, v: Self) -> Self {
                p_gcd(u, v)
            }
        }
    };
}
prim_int_util!(u8);
prim_int_util!(u16);
prim_int_util!(u32);
prim_int_util!(u64);
prim_int_util!(u128);

impl NumUtil for rug::Integer {
    fn gcd(u: Self, v: Self) -> Self {
        u.gcd(&v)
    }

    fn integer_square_root(self) -> Self {
        self.sqrt()
    }
}

#[cfg(test)]
mod tests {
    use crate::util::NumUtil;

    #[test]
    fn test_int_sqrt() {
        assert_eq!(4u64.integer_square_root(), 2);
        assert_eq!(15u64.integer_square_root(), 3);
        assert_eq!(16u64.integer_square_root(), 4);
        assert_eq!((1653u64 * 1653 - 1).integer_square_root(), 1652);
        assert_eq!((1653u64 * 1653).integer_square_root(), 1653);
        assert_eq!(u64::MAX.integer_square_root(), u64::from(u32::MAX));
    }
    #[test]
    fn test_gcd() {
        let mut v = 2u64;
        let mut u = 15_096_997_u64;
        for _ in 0u64..10000 {
            u = u.wrapping_mul(u).wrapping_add(8713);
            v = v.wrapping_mul(v).wrapping_add(4_891_895);

            let g = u64::gcd(u, v);
            assert_eq!(u % g, 0);
            assert_eq!(v % g, 0);
            assert_eq!(u64::gcd(u / g, v / g), 1);
        }
    }
}
