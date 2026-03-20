use std::ops::Add;
use std::ops::Sub;
use std::ops::Mul;
use std::ops::Div;
use std::ops::Neg;
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

    pub const ZERO: Self = Self { value: 0 };
    pub const ONE: Self = Self { value: 1 };

    // Computes (self ^ exponent) mod p using the square-and-multiply algorithm.
    // Runs in O(log(exponent)) time.
    // Crucial for computing the multiplicative inverse via Fermat's Little Theorem.
    pub fn pow(self, mut exponent: u64) -> Self {
        let mut res = Self::ONE;
        let mut base = self;

        while exponent > 0 {
            // If the least significant bit is 1, multiply the result by the current base
            if exponent & 1 == 1 {
                res = res * base; // Delegates to our Mul trait
            }
            
            // Square the base for the next bit
            base = base * base;
            
            // Shift the exponent right by 1 bit
            exponent >>= 1;
        }

        res
    }

    // Computes the multiplicative inverse of this element using Fermat's Little Theorem.
    // a^(p-1) ≡ 1 (mod p) => a^(-1) ≡ a^(p-2) (mod p).
    // Panics if trying to invert zero.
    pub fn inv(self) -> Self {
        if self.value == 0 {
            panic!("Attempted to divide by zero in Fp.");
        }
        
        // p - 2 = 0xfffffffeffffffff
        self.pow(0xfffffffeffffffff)
    }

    // Serializes the field element to a le byte array.
    pub fn to_bytes(self) -> [u8; 8] {
        self.value.to_le_bytes()
    }

    // Deserializes a field element from a le byte array.
    // Safely applies the canonical projection to ensure the value is strictly < p.
    pub fn from_bytes(bytes: [u8; 8]) -> Self {
        // We use Self::new() to guarantee the value is properly reduced mod p
        Self::new(u64::from_le_bytes(bytes))
    }
}

// Internal law of composition: Addition in the abelian group (Fp, +)
impl Add for Fp {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        // Perform 64-bit addition and catch the CPU carry flag
        let (sum, carry) = self.value.overflowing_add(rhs.value);

        // Branchless reduction:
        // hardware_overflow is 1 if sum >= 2^64, 0 otherwise.
        let hardware_overflow = carry as u64;
        
        // modular_overflow is 1 if p <= sum < 2^64, 0 otherwise.
        let modular_overflow = (sum >= MOD) as u64;
        
        // These two conditions are mutually exclusive. 
        // mask is 1 if we need to reduce, 0 otherwise.
        let mask = hardware_overflow | modular_overflow;

        // Affine projection: add (2^32 - 1) if mask is 1, add 0 if mask is 0.
        // This effectively simulates (sum - p) when an overflow occurs.
        let corrected_sum = sum.wrapping_add(0xffffffff * mask);

        Self { value: corrected_sum }
    }
}

// Internal law of composition: Subtraction in the abelian group (Fp, +)
impl Sub for Fp {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        // Perform 64-bit subtraction and catch the CPU borrow flag
        let (diff, borrow) = self.value.overflowing_sub(rhs.value);

        // Branchless correction:
        // If a < b, the CPU wrapped around and added 2^64.
        // borrow as u64 is 1 if underflow, 0 otherwise.
        let mask = borrow as u64;
        
        // Since 2^64 ≡ 2^32 - 1 (mod p), we must subtract (2^32 - 1) to correct it.
        // We subtract it only if mask is 1.
        let corrected_diff = diff.wrapping_sub(0xffffffff * mask);

        Self { value: corrected_diff }
    }
}
    
// Multiplication in the finite field Fp
// This implementation heavily relies on the highly optimized reduction algorithm 
// found in the Plonky2 repository.
// Mathematical background for 128-bit reduction without Euclidean division:
// Let p = 2^64 - 2^32 + 1. Thus, 2^64 ≡ 2^32 - 1 (mod p).
// Let E = 2^32 - 1 (Epsilon).
// A 128-bit product X can be written as: X = H * 2^64 + L
// Therefore, X ≡ H * E + L (mod p).
// 
// To avoid overflow when computing H * E, Plonky2 splits H into 32-bit halves:
// H = H_hi * 2^32 + H_lo
// X ≡ (H_hi * 2^32 + H_lo) * E + L (mod p)
// X ≡ H_hi * 2^64 - H_hi * 2^32 + H_lo * E + L (mod p)
// 
// Since 2^32 * E = 2^64 - 2^32 ≡ E - 2^32 = -1 (mod p),
// we get H_hi * 2^32 * E ≡ -H_hi (mod p).
// 
// The magic cancellation happens here:
// X ≡ (H_hi * 2^32 - H_hi) - H_hi * 2^32 + H_lo * E + L (mod p)
// X ≡ L - H_hi + H_lo * E (mod p)
// 
// We compute this exact equation in 3 steps below.
impl Mul for Fp {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        // 1. 128-bit hardware multiplication
        let prod = (self.value as u128) * (rhs.value as u128);

        // 2. Split into 64-bit halves (Extract L and H)
        let x_lo = prod as u64;           // L
        let x_hi = (prod >> 64) as u64;   // H

        // 3. Split the high part into two 32-bit halves
        let x_hi_hi = x_hi >> 32;         // H_hi
        let x_hi_lo = x_hi & 0xffffffff;  // H_lo

        // 4. Step A: t0 = L - H_hi
        let (mut t0, borrow) = x_lo.overflowing_sub(x_hi_hi);
        
        // Rare underflow correction path.
        if borrow {
            t0 = t0.wrapping_sub(0xffffffff);
        }

        // 5. Step B: t1 = H_lo * E
        // Both operands are 32-bit, so their product perfectly fits in a 64-bit 
        // register without any risk of hardware overflow.
        let t1 = x_hi_lo * 0xffffffff;

        // 6. Step C: Final addition (t0 + t1)
        // We delegate the final reduction to our constant-time, branchless Add trait.
        // It guarantees the result lands exactly in the canonical [0, p-1] range.
        Self { value: t0 } + Self { value: t1 }
    }
}

impl Div for Fp {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        // rhs.inv() will panic automatically if rhs is zero.
        // We then multiply by the inverse using our optimized Mul trait.
        self * rhs.inv()
    }
}

impl Neg for Fp {
    type Output = Self;

    fn neg(self) -> Self::Output {
        // Delegates to our constant-time, branchless Sub trait
        Self::ZERO - self
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

    // Tests generated by Gemini to strictly verify branchless logic and algebraic edge cases
    #[test]
    fn test_add_branchless_boundaries() {
        let a = Fp { value: 10 };
        let b = Fp { value: 20 };
        assert_eq!((a + b).value, 30);

        let p_minus_one = 0xffffffff00000000;
        let a = Fp { value: p_minus_one };
        let b = Fp { value: 2 };
        assert_eq!((a + b).value, 1);

        let a = Fp { value: p_minus_one };
        let b = Fp { value: p_minus_one };
        assert_eq!((a + b).value, 0xfffffffeffffffff);
    }

    #[test]
    fn test_sub_branchless_boundaries() {
        let a = Fp { value: 42 };
        let b = Fp { value: 12 };
        assert_eq!((a - b).value, 30);

        let a = Fp { value: 100 };
        let b = Fp { value: 100 };
        assert_eq!((a - b).value, 0);

        let a = Fp { value: 0 };
        let b = Fp { value: 1 };
        let p_minus_one = 0xffffffff00000000;
        assert_eq!((a - b).value, p_minus_one);
    }

    #[test]
    fn test_algebraic_properties() {
        let a = Fp { value: 0xffffffff00000000 };
        let b = Fp { value: 0x123456789abcdef0 };

        let sum = a + b;
        let diff = sum - b;
        assert_eq!(diff.value, a.value);

        assert_eq!((a + b).value, (b + a).value);
    }

    // Mul tests
    // These tests were generated by Gemini to strictly verify the 128-bit reduction logic
    // and the fundamental axioms of the finite field.

    #[test]
    fn test_mul_basic() {
        let a = Fp { value: 10 };
        let b = Fp { value: 20 };
        assert_eq!((a * b).value, 200);
    }

    #[test]
    fn test_mul_zero_and_one() {
        let a = Fp { value: 42 };
        let zero = Fp { value: 0 };
        let one = Fp { value: 1 };

        // Absorbing element
        assert_eq!((a * zero).value, 0);
        // Identity element
        assert_eq!((a * one).value, 42);
    }

    #[test]
    fn test_mul_max_values() {
        // The ultimate 128-bit reduction crash test.
        // Mathematically: (p - 1) * (p - 1) ≡ (-1) * (-1) ≡ 1 (mod p)
        let p_minus_one = 0xffffffff00000000;
        let a = Fp { value: p_minus_one };
        let b = Fp { value: p_minus_one };
        
        assert_eq!((a * b).value, 1);
    }

    #[test]
    fn test_mul_commutativity() {
        let a = Fp { value: 0x123456789abcdef0 };
        let b = Fp { value: 0x0fedcba987654321 };
        
        // a * b = b * a
        assert_eq!((a * b).value, (b * a).value);
    }

    #[test]
    fn test_ring_distributivity() {
        // Verifies that multiplication distributes over our branchless addition.
        // a * (b + c) = (a * b) + (a * c)
        let a = Fp { value: 100 };
        let b = Fp { value: 0xffffffff00000000 }; // p - 1
        let c = Fp { value: 5 };
        
        let left = a * (b + c);
        let right = (a * b) + (a * c);
        
        assert_eq!(left.value, right.value);
    }

    // // These tests were generated by Gemini

    #[test]
    fn test_pow_fermat() {
        // Fermat's Little Theorem: a^(p-1) ≡ 1 (mod p)
        // This validates both our pow algorithm and our 128-bit multiplication under extreme loads.
        let p_minus_one = 0xffffffff00000000;
        let a = Fp { value: 42 }; // Arbitrary non-zero element
        
        assert_eq!(a.pow(p_minus_one).value, 1);
    }

    #[test]
    fn test_inverse() {
        let a = Fp { value: 42 };
        let inv_a = a.inv();
        
        // a * a^(-1) ≡ 1 (mod p)
        assert_eq!((a * inv_a).value, 1);
    }

    #[test]
    #[should_panic(expected = "Attempted to divide by zero")]
    fn test_inverse_zero() {
        let zero = Fp { value: 0 };
        zero.inv(); // This should panic
    }

    // These tests were generated by Gemini

    #[test]
    fn test_div_basic() {
        let a = Fp { value: 100 };
        let b = Fp { value: 2 };
        
        // 100 / 2 = 50
        assert_eq!((a / b).value, 50);
    }

    #[test]
    fn test_div_advanced() {
        let a = Fp { value: 42 };
        let b = Fp { value: 7 };
        let c = a / b;
        
        // c * b should get us back to a
        assert_eq!((c * b).value, a.value);
    }

    #[test]
    #[should_panic(expected = "Attempted to divide by zero")]
    fn test_div_by_zero() {
        let a = Fp { value: 42 };
        let zero = Fp { value: 0 };
        
        // This must panic
        let _ = a / zero;
    }

    // These tests were generated by Gemini
    
    #[test]
    fn test_negation() {
        let a = Fp::new(42);
        let neg_a = -a;
        
        // a + (-a) = 0
        assert_eq!((a + neg_a).value(), 0);

        // -0 = 0
        let zero = Fp::ZERO;
        assert_eq!((-zero).value(), 0);
    }

    #[test]
    fn test_serialization() {
        let a = Fp::new(0x123456789abcdef0);
        let bytes = a.to_bytes();
        let b = Fp::from_bytes(bytes);
        
        assert_eq!(a, b);
    }
}