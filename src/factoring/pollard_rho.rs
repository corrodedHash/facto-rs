use redc::Redc;
use redc::{self, Field};
use twoword::TwoWord;

use crate::util::NumUtil;

use super::brent_cycle::find_cycle;

/// Find factors of given number by applying Pollard's rho algorithm using Brent's cycle detection
///
/// <https://en.wikipedia.org/wiki/Pollard%27s_rho_algorithm>
pub trait PollardRho: Sized {
    /// Factorize given number
    ///
    /// Generates series of `x` = `x` * `x` + `increment`.
    /// The first x of this series is `start`
    ///
    /// # Returns
    /// A factor if one has been found, or `None` if the algorithm was unsuccessful
    fn pollard_rho(self, start: &Self, increment: &Self) -> Option<Self>;
}

struct PollardRhoCycleConditionCheckerU64 {
    field: <u64 as Redc>::FieldType,
    accum: u64,
    n: u64,
    last_tortoise: u64,
    last_hare: u64,
}

impl super::brent_cycle::CycleConditionChecker<u64, u64> for PollardRhoCycleConditionCheckerU64 {
    fn check(&mut self, tortoise: &u64, hare: &u64, count: &u64, power: &u64) -> bool {
        let diff = if hare > tortoise {
            hare - tortoise
        } else {
            tortoise - hare
        };
        self.accum = self.field.redc(u128::from(self.accum) * u128::from(diff));
        self.last_tortoise = *tortoise;
        debug_assert_eq!(power.count_ones(), 1);
        let power_count = power.trailing_zeros();

        if (power - count) & ((1 << std::cmp::max(5, power_count / 2)) - 1) == 1 {
            let d = u64::gcd(self.accum.to_normal(&self.field), self.n);
            if d != 1 {
                return true;
            }
            self.last_hare = *hare;
        }
        false
    }
}

impl PollardRhoCycleConditionCheckerU64 {
    fn new(field: &<u64 as Redc>::FieldType, n: u64, start: u64) -> Self {
        Self {
            field: field.clone(),
            accum: 1u64.to_montgomery_unchecked(field),
            n,
            last_tortoise: start,
            last_hare: start,
        }
    }
    fn extract(self, mut f: PollardRhoMapperU64) -> u64 {
        let mut hare = super::brent_cycle::MapFunction::run(&mut f, self.last_hare);
        #[allow(unused_mut, unused_variables, dead_code)]
        loop {
            let x_minus_y_abs = if hare > self.last_tortoise {
                hare - self.last_tortoise
            } else {
                self.last_tortoise - hare
            };
            let d = u64::gcd(x_minus_y_abs.to_normal(&self.field), self.n);
            if d != 1 {
                return d;
            }
            hare = super::brent_cycle::MapFunction::run(&mut f, hare);
        }
    }
}

struct PollardRhoMapperU64(u64, <u64 as Redc>::FieldType);
impl super::brent_cycle::MapFunction<u64> for PollardRhoMapperU64 {
    fn run(&mut self, n: u64) -> u64 {
        self.1
            .redc(u128::from(n) * u128::from(n) + u128::from(self.0))
    }
}

struct PollardRhoCycleConditionCheckerU128 {
    field: <u128 as Redc>::FieldType,
    accum: u128,
    n: u128,
    last_tortoise: u128,
    last_hare: u128,
}

impl super::brent_cycle::CycleConditionChecker<u128, u128> for PollardRhoCycleConditionCheckerU128 {
    #[inline]
    fn check(&mut self, tortoise: &u128, hare: &u128, count: &u128, power: &u128) -> bool {
        let diff = if hare > tortoise {
            hare - tortoise
        } else {
            tortoise - hare
        };
        self.accum = self.field.redc(TwoWord::mult(self.accum, diff));
        self.last_tortoise = *tortoise;
        debug_assert_eq!(power.count_ones(), 1);
        let power_count = power.trailing_zeros();

        if (power - count) & ((1 << std::cmp::max(5, power_count / 2)) - 1) == 1 {
            let d = u128::gcd(self.accum.to_normal(&self.field), self.n);
            if d != 1 {
                return true;
            }
            self.last_hare = *hare;
        }
        false
    }
}

impl PollardRhoCycleConditionCheckerU128 {
    fn new(field: &<u128 as Redc>::FieldType, n: u128, start: u128) -> Self {
        Self {
            field: field.clone(),
            accum: 1u128.to_montgomery_unchecked(field),
            n,
            last_tortoise: start,
            last_hare: start,
        }
    }
    fn extract(self, mut f: PollardRhoMapperU128) -> u128 {
        let mut hare = super::brent_cycle::MapFunction::run(&mut f, self.last_hare);
        loop {
            let x_minus_y_abs = if hare > self.last_tortoise {
                hare - self.last_tortoise
            } else {
                self.last_tortoise - hare
            };
            let d = u128::gcd(x_minus_y_abs.to_normal(&self.field), self.n);
            if d != 1 {
                return d;
            }
            hare = super::brent_cycle::MapFunction::run(&mut f, hare);
        }
    }
}

struct PollardRhoMapperU128(u128, <u128 as Redc>::FieldType);
impl super::brent_cycle::MapFunction<u128> for PollardRhoMapperU128 {
    fn run(&mut self, n: u128) -> u128 {
        self.1.redc(
            TwoWord::mult(n, n)
                + TwoWord {
                    lower: self.0,
                    higher: 0,
                },
        )
    }
}

struct PollardRhoCycleConditionCheckerRug {
    field: <rug::Integer as Redc>::FieldType,
    accum: rug::Integer,
    n: rug::Integer,
    last_tortoise: rug::Integer,
    last_hare: rug::Integer,
}

impl PollardRhoCycleConditionCheckerRug {
    #[inline]
    fn check(
        &mut self,
        tortoise: &rug::Integer,
        hare: &rug::Integer,
        count: &rug::Integer,
        power: &rug::Integer,
    ) -> bool {
        let diff = if hare > tortoise {
            hare.clone() - tortoise
        } else {
            tortoise.clone() - hare
        };
        self.accum = self.field.redc(self.accum.clone() * diff);
        self.last_tortoise = tortoise.clone();
        debug_assert_eq!(power.count_ones(), Some(1));
        let power_count = power.find_one(0).unwrap();

        if (power.clone() - count).keep_bits(std::cmp::max(4, power_count / 2)) == 1 {
            let d = self.accum.clone().to_normal(&self.field).gcd(&self.n);
            if d != 1 {
                return true;
            }
            self.last_hare = hare.clone();
        }
        false
    }
    fn new(
        field: &<rug::Integer as Redc>::FieldType,
        n: rug::Integer,
        start: rug::Integer,
    ) -> Self {
        Self {
            field: field.clone(),
            accum: rug::Integer::from(1).to_montgomery_unchecked(field),
            n,
            last_tortoise: start.clone(),
            last_hare: start,
        }
    }
    fn extract(
        self,
        increment: &rug::Integer,
        field: &<rug::Integer as Redc>::FieldType,
    ) -> rug::Integer {
        let mut hare: rug::Integer = field.redc(self.last_hare.square() + increment);
        #[allow(unused_mut, unused_variables, dead_code)]
        loop {
            let x_minus_y_abs: rug::Integer = if hare > self.last_tortoise {
                hare.clone() - &self.last_tortoise
            } else {
                self.last_tortoise.clone() - &hare
            };
            let d = x_minus_y_abs.to_normal(&self.field).gcd(&self.n);
            if d != 1 {
                return d;
            }
            hare = field.redc(hare.square() + increment);
        }
    }
}

impl PollardRho for u64 {
    fn pollard_rho(self, start: &Self, constant_increment: &Self) -> Option<Self> {
        let field = self.setup_field();
        let start = start.to_montgomery(&field);
        let constant_increment = constant_increment.to_montgomery(&field);
        #[allow(unused_mut, unused_variables, dead_code)]
        let (e, _) = find_cycle::<_, Self, _, _>(
            PollardRhoMapperU64(constant_increment, field.clone()),
            PollardRhoCycleConditionCheckerU64::new(&field, self, start),
            start,
        );
        let d = e.extract(PollardRhoMapperU64(constant_increment, field));
        if d == self {
            None
        } else {
            Some(d)
        }
    }
}

impl PollardRho for u128 {
    fn pollard_rho(self, start: &Self, constant_increment: &Self) -> Option<Self> {
        let field = self.setup_field();
        let start = start.to_montgomery(&field);
        let constant_increment = constant_increment.to_montgomery(&field);
        #[allow(unused_mut, unused_variables, dead_code)]
        let (e, _) = find_cycle::<_, Self, _, _>(
            PollardRhoMapperU128(constant_increment, field.clone()),
            PollardRhoCycleConditionCheckerU128::new(&field, self, start),
            start,
        );
        let d = e.extract(PollardRhoMapperU128(constant_increment, field));
        if d == self {
            None
        } else {
            Some(d)
        }
    }
}

fn find_rug_cycle(
    mut cycle_condition: PollardRhoCycleConditionCheckerRug,
    start: rug::Integer,
    increment: &rug::Integer,
    field: &<rug::Integer as Redc>::FieldType,
) -> PollardRhoCycleConditionCheckerRug {
    let mut tortoise = start.clone();
    let mut hare = field.redc(start.square() + increment);
    let mut power = rug::Integer::from(1);
    let mut count = rug::Integer::from(0);
    while !cycle_condition.check(&tortoise, &hare, &count, &power) {
        count += 1;
        if power == count {
            tortoise = hare.clone();
            power <<= 1;
            count = rug::Integer::from(0);
        }
        hare = field.redc(hare.square() + increment);
    }
    cycle_condition
}

impl PollardRho for rug::Integer {
    fn pollard_rho(self, start: &Self, constant_increment: &Self) -> Option<Self> {
        let field = self.clone().setup_field();
        let start = start.clone().to_montgomery(&field);
        let constant_increment = constant_increment.clone().to_montgomery(&field);
        #[allow(unused_mut, unused_variables, dead_code)]
        let e = find_rug_cycle(
            PollardRhoCycleConditionCheckerRug::new(&field, self.clone(), start.clone()),
            start,
            &constant_increment,
            &field,
        );
        let d = e.extract(&constant_increment, &field);
        if d == self {
            None
        } else {
            Some(d)
        }
    }
}
