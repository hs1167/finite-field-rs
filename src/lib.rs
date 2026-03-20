use std::ops::Add;
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

// Internal law of composition: Addition in the abelian group (Fp, +)
impl Add for Fp {
    type Output = Self;

    fn add(self,right: Self) -> Self::Output {
        // Perform 64-bit addition and catch the CPU carry flag to manage the overflow
        let (mut sum, overflow) = self.value.overflowing_add(right.value);
        
        // Check if the result is outside the canonical range [0, MOD - 1]
        if overflow || sum >= MOD {
            // a pretty cool trick:
            // we have p = 2^64 - 2^32 + 1, thus 2^64 ≡ 2^32 - 1 (mod p)
            // If we overflowed (sum >= 2^64), we replace the missing 2^64 with 2^32 - 1.
            // If we are between p and 2^64, adding 2^32 - 1 effectively 
            //   performs (sum - p) due to the 64-bit wrap-around.
            let (new_sum, _) = sum.overflowing_add(0xffff_ffff); 
            sum = new_sum;
        }

        Self { value: sum }
    }
}

    
    


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_projection() {
        // Test standard value
        assert_eq!(Fp::new(100).value(), 100);

        // Test p ≡ 0 (mod p)
        assert_eq!(Fp::new(MOD).value(), 0);

        // Test p + 1 ≡ 1 (mod p)
        assert_eq!(Fp::new(MOD + 1).value(), 1);
        
        // Test maximum u64 value
        assert_eq!(Fp::new(u64::MAX).value(), u64::MAX % MOD);
    }

    #[test]
    fn test_addition_basic() {
        // Standard addition without reduction
        let a = Fp::new(10);
        let b = Fp::new(20);
        assert_eq!((a + b).value(), 30);

        // Neutral element: a + 0 = a
        let a = Fp::new(MOD - 1);
        let b = Fp::new(0);
        assert_eq!((a + b).value(), MOD - 1);
    }

    #[test]
    fn test_addition_p_boundary() {
        // Case: p <= sum < 2^64 (Triggers the 'sum >= MOD' condition)
        // (p - 1) + 2 = p + 1 ≡ 1 (mod p)
        let a = Fp::new(MOD - 1);
        let b = Fp::new(2);
        assert_eq!((a + b).value(), 1);

        // (p - 1) + (p - 1) = 2p - 2 ≡ p - 2 (mod p)
        let a = Fp::new(MOD - 1);
        let b = Fp::new(MOD - 1);
        assert_eq!((a + b).value(), MOD - 2);
    }

    #[test]
    fn test_addition_overflow_64bit() {
        // Case: sum >= 2^64 (Triggers the 'overflow' flag)
        // We use the property 2^64 ≡ 2^32 - 1 (mod p)
        let a = Fp::new(u64::MAX); // 2^64 - 1
        let b = Fp::new(1);
        
        // (2^64 - 1) + 1 = 2^64 ≡ 2^32 - 1 (mod p)
        // 2^32 - 1 is 0xffff_ffff
        assert_eq!((a + b).value(), 0xffff_ffff);
    }
}