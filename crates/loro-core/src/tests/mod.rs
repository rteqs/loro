#![cfg(test)]

#[cfg(feature = "fuzzing")]
pub const PROPTEST_FACTOR_10: usize = 10;
#[cfg(not(feature = "fuzzing"))]
pub const PROPTEST_FACTOR_10: usize = 1;
