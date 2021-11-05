#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct BitVector {
    elements: Vec<u128>,
}
impl std::fmt::Debug for BitVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BitVector {{\n\t")?;
        for x in &self.elements {
            write!(f, "{:032X}", x)?;
        }
        write!(f, "\n}}")?;
        Ok(())
    }
}

impl BitVector {
    pub fn new(size: usize) -> Self {
        Self {
            elements: vec![0; (size + 127) / 128],
        }
    }
    pub fn trailing_zeros(&self) -> usize {
        let mut r = 0usize;
        for e in &self.elements {
            if 0u128.trailing_zeros() != e.trailing_zeros() {
                return r + e.trailing_zeros() as usize;
            }
            r += 0u128.trailing_zeros() as usize;
        }
        r
    }
    pub fn is_zero(&self) -> bool {
        self.elements.iter().all(|x| x == &0)
    }
    pub fn add(&mut self, other: &Self) {
        for (s, o) in self.elements.iter_mut().zip(other.elements.iter()) {
            *s ^= o;
        }
    }
    pub const fn bit_helper(index: usize) -> (u128, usize) {
        let bit_index = index % 128;
        let cell_index = index / 128;
        let mask = 1u128 << bit_index;
        (mask, cell_index)
    }
    pub fn set(&mut self, index: usize, value: bool) {
        let (mask, cell_index) = Self::bit_helper(index);
        let cell = self
            .elements
            .get_mut(cell_index)
            .expect("Index out of bounds");
        if value {
            *cell |= mask;
        } else {
            *cell &= !mask;
        }
    }
    pub fn get(&self, index: usize) -> bool {
        let (mask, cell_index) = Self::bit_helper(index);
        let cell = self.elements.get(cell_index).expect("Index out of bounds");
        (*cell & mask) != 0
    }
    pub fn flip(&mut self, index: usize) {
        let (mask, cell_index) = Self::bit_helper(index);
        let cell = self
            .elements
            .get_mut(cell_index)
            .expect("Index out of bounds");
        *cell ^= mask;
    }
}
