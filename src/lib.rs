// Goldilocks field: 2^64 - 2^32 + 1
// hexadecimal base: 0xffffffff00000001 with '0x' prefixe for hexadecimal base
pub const MOD: u64 = 0xffff_ffff_0000_0001; 

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fp {
    value: u64, // without pub to ensure the safety of wrapper
}

impl Fp {
    // canonical projection  p : Z -> Fp
    pub fn new(val: u64) -> Self {
        Self {
            value: val % MOD,
        }
    }

    
    pub fn value(self) -> u64 { // getter / read only
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Basic algebraic tests
    #[test]
    fn test_creation_and_modulo() {

        let a = Fp::new(100);
        assert_eq!(a.value(), 100);

        let b = Fp::new(MOD);
        assert_eq!(b.value(), 0);

        let c = Fp::new(MOD + 1);
        assert_eq!(c.value(), 1);
    }
}