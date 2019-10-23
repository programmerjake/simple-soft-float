// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

#[cfg(test)]
mod tests {
    use algebraics::algebraic_numbers::RealAlgebraicNumber;
    use num_bigint::BigInt;

    #[test]
    fn it_works() {
        assert_eq!(
            RealAlgebraicNumber::from(2) + RealAlgebraicNumber::from(2),
            RealAlgebraicNumber::from(BigInt::from(4))
        );
    }
}

macro_rules! doctest {
    ($x:expr) => {
        #[doc = $x]
        extern {}
    };
}

doctest!(include_str!("../README.md"));
