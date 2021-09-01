pub trait CycleConditionChecker<T, CountType> {
    fn check(&mut self, tortoise: &T, hare: &T, count: &CountType, power: &CountType) -> bool;
}

pub trait MapFunction<T> {
    fn run(&mut self, n: T) -> T;
}

pub fn find_cycle<T, CountType, Mapper, ConditionChecker>(
    mut mapper: Mapper,
    mut cycle_condition: ConditionChecker,
    start: T,
) -> (ConditionChecker, CountType)
where
    T: Clone + std::fmt::Debug,
    CountType: num_traits::PrimInt + std::ops::ShlAssign + std::ops::AddAssign,
    Mapper: MapFunction<T>,
    ConditionChecker: CycleConditionChecker<T, CountType>,
{
    let mut tortoise = start.clone();
    let mut hare = mapper.run(start);
    let mut power = CountType::one();
    let mut count = CountType::zero();
    while !cycle_condition.check(&tortoise, &hare, &count, &power) {
        count += CountType::one();
        if power == count {
            tortoise = hare.clone();
            power <<= CountType::one();
            count = CountType::zero();
        }
        hare = mapper.run(hare);
    }
    (cycle_condition, count)
}
