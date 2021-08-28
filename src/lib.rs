#![no_std]

use core::{
    fmt::Debug,
    cmp::Ord,
    ops::{Add, Sub, Mul, Div, Rem, Shr, Shl}
};

const SCALE_BITS: i32 = 12;
const SCALE_INV: i32 = 32 - SCALE_BITS;
const FRAC_BITS: i8 = SCALE_INV as i8;

#[inline(always)]
fn shift<S>(x: S, s: i8) -> S where
    S: Shr<i8, Output = S> + Shl<i8, Output = S>
{
    if s < 0 {
        x << -s
    } else {
        x >> s
    }
}

pub trait Scaled {
    fn value(&self) -> i32;

    fn frac_bits(&self) -> i8;
}

#[derive(Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Debug)]
pub struct Fixed{
    value: i32
}

impl Scaled for Fixed {
    #[inline(always)] // constant
    fn frac_bits(&self) -> i8 {
        FRAC_BITS as i8
    }

    #[inline(always)] // field access
    fn value(&self) -> i32 {
        self.value
    }
}

#[derive(Copy, Clone, Debug)]
pub struct GivenScale{
    pub raw_value: i32,
    pub frac_bits: i8
}

impl Scaled for GivenScale {
    #[inline(always)] // field access
    fn frac_bits(&self) -> i8 {
        self.frac_bits as i8
    }

    #[inline(always)] // field access
    fn value(&self) -> i32 {
        self.raw_value
    }
}

macro_rules! scaled_int {
    ($($T:ty),+) => {
        $(
            impl Scaled for $T {
                #[inline(always)] // constant
                fn frac_bits(&self) -> i8 {
                    0
                }
            
                #[inline(always)] // identity
                fn value(&self) -> i32 {
                    *self as i32
                }
            }
        )+
    }
}

scaled_int!{i32, u32, i16, u16, i8, u8}

impl Rem for Fixed{
    type Output = Self;
    fn rem(self, modulus: Self) -> Self::Output {
        Fixed{value: self.value % modulus.value}
    }
}

#[inline(always)]
fn add_0(l: i32, lb: i8, r: i32, rb: i8) -> Fixed{
    Fixed{value: shift(l, lb - FRAC_BITS) + shift(r, rb - FRAC_BITS)}
}

#[inline(always)]
fn sub_0(l: i32, lb: i8, r: i32, rb: i8) -> Fixed{
    Fixed{value: shift(l, lb - FRAC_BITS) - shift(r, rb - FRAC_BITS)}
}

#[inline(always)]
fn mul_0(l: i32, lb: i8, r: i32, rb: i8) -> Fixed{
    let intermediate: i64 = l as i64 * r as i64;
    let sh =  lb + rb - FRAC_BITS;
    Fixed{value: shift(intermediate, sh) as i32}
}

#[inline(always)]
fn div_0(l: i32, lb: i8, r: i32, rb: i8) -> Fixed{
    let numerator = (l as i64) << 32;
    let intermediate = numerator / r as i64;
    let div_frac_bits = lb + 32 - rb;
    let sh =  div_frac_bits - FRAC_BITS;
    Fixed{value: shift(intermediate, sh) as i32}
}

macro_rules! fixed_math {
    ( $( ($LHS:ty, $RHS:ty) ),+ )
    => { $(
        macro_rules! fixed_math_opr {
            ($Opr:ident, $op:ident, $op0:ident) => {
                impl $Opr<$RHS> for $LHS {
                    type Output = Fixed;
                    fn $op(self, o: $RHS)-> Fixed{
                        $op0(self.value(), self.frac_bits(), o.value(), o.frac_bits())
                    }
                }
            }
        }
        fixed_math_opr!{Add, add, add_0}
        fixed_math_opr!{Sub, sub, sub_0}
        fixed_math_opr!{Mul, mul, mul_0}
        fixed_math_opr!{Div, div, div_0}
    )+ }
}

fixed_math!{
    (Fixed, Fixed),
    (GivenScale, GivenScale),
    (GivenScale, Fixed), (Fixed, GivenScale),
    (i32, Fixed), (Fixed, i32),
    (u32, Fixed), (Fixed, u32),
    (i16, Fixed), (Fixed, i16),
    (u16, Fixed), (Fixed, u16),
    ( i8, Fixed), (Fixed, i8),
    ( u8, Fixed), (Fixed, u8),
    (i32, GivenScale), (GivenScale, i32),
    (u32, GivenScale), (GivenScale, u32),
    (i16, GivenScale), (GivenScale, i16),
    (u16, GivenScale), (GivenScale, u16),
    ( i8, GivenScale), (GivenScale, i8),
    ( u8, GivenScale), (GivenScale, u8)
}


impl Fixed {
    pub fn to_f32(&self) -> f32{
        self.value as f32 / ((1 << SCALE_INV) as f32)
    }

    pub fn to_i32(&self) -> i32{
        self.value >> SCALE_INV
    }

    pub const fn from_i32(i: i32) -> Fixed{
        Fixed{value: i << SCALE_INV}
    }

    pub fn from_int<F>(i: F) -> Fixed where F: Into<i32>{
        Fixed{value: i.into() << SCALE_INV}
    }

    pub const fn from_raw(value: i32) -> Fixed{
        Fixed{value}
    }

    pub fn from_decimal(whole: i32, decimal_points: u16) -> Fixed{
        let d64 = (whole as u64) << 32;
        let mut decimal_div = 1u64;
        for _ in 0..decimal_points {
            decimal_div *= 10;
        }

        Fixed{value: ((d64 / decimal_div) >> SCALE_BITS) as i32}
    }

    pub const ZERO: Fixed = Fixed{value: 0};
    pub const ONE: Fixed = Fixed{value: 1 << SCALE_INV};

    pub fn clamp_i32(self, min: i32, max: i32) -> Fixed{
        if min > max {
            panic!("min > max");
        }

        let fmin = Fixed::from_i32(min);
        let fmax = Fixed::from_i32(max);
        if  fmin > self {
            return fmin;
        } else if fmax < self {
            return fmax;
        }else{
            return self;
        }
    }

    pub fn negate(self) -> Fixed{
        Fixed{value: -self.value}
    }

    pub fn abs(self) -> Fixed{
        Fixed{value: self.value.abs()}
    }

    pub const HALF :Fixed = Fixed::from_raw(524288);
    pub const HPI  :Fixed = Fixed::from_raw(1647099);
    pub const E    :Fixed = Fixed::from_raw(2850325);
    pub const PI   :Fixed = Fixed::from_raw(3294199);
    pub const TAU  :Fixed = Fixed::from_raw(6588397);

    pub fn sin_dimless(self: Fixed) -> Fixed{
        let flip = self > Fixed::HALF;
        
        // The first half is more accurate than the second
        let z = match flip {
            true => self - Fixed::HALF,
            _ => self
        };
        
        // and the first quarter is even more accurate
        const QUARTER:Fixed = Fixed{value: 262144};
        let z = match z >= QUARTER {
            true => Fixed::HALF-z,
            _ => z
        };

        let z = Fixed{value: z.value << 2}; // the function was designed for a domain of 0-4
        let hz = Fixed{value: z.value >> 1};
        let zsq = z * z;
        const PIM3:Fixed = Fixed{value: 148471};
        const TAUM5:Fixed = Fixed{value: 1345517};

        let ret = hz * (Fixed::PI - (zsq * (TAUM5 - (zsq * PIM3))));

        match flip {
            true => ret.negate(),
            _ => ret
        }
    }

    pub fn sin(self: Fixed) -> Fixed{
        let flip = self > Fixed::PI;
        
        // The first half is more accurate than the second
        let t = match flip {
            true => self - Fixed::PI,
            _ => self
        };

        // and the first quarter is even more accurate
        let t = match t >= Fixed::HPI {
            true => Fixed::PI-t,
            _ => t
        };

        // 1/(2*3), 1/(4*5)...
        const I23   :Fixed = Fixed::from_raw(174763);
        const I45   :Fixed = Fixed::from_raw(52429);
        const I67   :Fixed = Fixed::from_raw(24966);
        const I89   :Fixed = Fixed::from_raw(14564);
    
    
        let t2 = t * t;
        let t3 = t * t2;
    
        let t2_45   = t2 * I45;  // t^2 / 4*5
        let t2_67   = t2 * I67;  // t^2 / 6*7
        let t2_89   = t2 * I89;  // etc
        
        let t3i  = t3  * I23;    // t^3 / 2*3
        let t5i  = t3i  * t2_45; // (t^3 * t^2) / (2*3 * 4*5)
        let t7i  = t5i  * t2_67; // etc
        let t9i  = t7i  * t2_89;
    
        let ret = t - t3i + t5i - t7i + t9i;
    
        match flip {
            true => ret.negate(),
            _ => ret
        }
    }

    pub fn inverse(self) -> Fixed {
        if self == Fixed::ZERO {
            return Fixed::ZERO;
        }

        let negative = self < Fixed::ZERO;
        let x = self.abs();

        if x == Fixed::ONE {
            return match negative {
                true => Fixed::ONE.negate(),
                _    => Fixed::ONE
            }
        }

        if x.value <= (1 << (SCALE_INV - SCALE_BITS)){
            // to big, saturate
            let max = Fixed::from_raw(i32::MAX);
            return match negative {
                true => max.negate(),
                _    => max
            }
        }

        if x > Fixed::ONE{
            let mut n = 0;  
            // Just get the whole part of the number
            let whole = x.value >> SCALE_INV;
            // find the power of two
            while whole >> n > 1 {
                n += 1;
            }

            // Round up
            if whole - (whole >> 1) > 0 {
                n += 1;
            }

            // We will use a taylor series centered around this point
            // let taylor_pt = 1 << n;
            // 1 / taylor_pt
            let taylor_i = Fixed{value: 1 << (SCALE_INV - n)};

            // Multiplicitive basis, each term is multiplied by this to get the next term
            let mul_basis = (taylor_i.negate() * x) + 1;

            let mut acc = Fixed::ZERO;  // result accumulator
            let mut mul_acc = taylor_i; // term accummulator

            let mut n = 0;
            while n < 11 && mul_acc.value > 100{
                acc = acc + mul_acc;
                mul_acc = mul_acc * mul_basis;
                n += 1;
            }

            return match negative {
                true => acc.negate(),
                _    =>  acc
            };
        } else {
            let mut guess = Fixed::ONE;
            let one_bits = Fixed::ONE.value;
            let mut fractional = x.value << 1;
            while fractional < one_bits {
                guess = Fixed{value: guess.value << 1};
                fractional = fractional << 1;
            }

            for _ in 0..8{
                let next: Fixed = guess * (2 - (x * guess));
                if (next.value - guess.value).abs() < 100{
                    return next;
                }
                guess = next;
            }
            panic!("failed to converge");
        }
    }

    pub fn inv_i32(denominator: i32) -> Fixed{
        if denominator == 0 {
            return Fixed::ZERO;
        }

        let negative = denominator < 0;
        let x = denominator.abs();

        if x == 1 {
            return match negative {
                true => Fixed::ONE.negate(),
                _    => Fixed::ONE
            }
        }

        let mut n = 0;  
        // find the power of two
        while x >> n > 1 {
            n += 1;
        }

        // Round up
        if x - (x >> 1) > 0 {
            n += 1;
        }

        if n > SCALE_INV {
            return Fixed::ZERO;
        }

        // We will use a taylor series centered around this point
        // let taylor_pt = 1 << n;

        // 1 / taylor_pt
        let taylor_i = Fixed{value: 1 << (SCALE_INV - n)};

        // Multiplicitive basis, each term is multiplied by this to get the next term
        let mul_basis = (taylor_i.negate() * x) + 1;

        let mut acc = Fixed::ZERO;  // result accumulator
        let mut mul_acc = taylor_i; // term accummulator

        let mut n = 0;
        while n < 11 && mul_acc.value > 100{
            acc = acc + mul_acc;
            mul_acc = mul_acc * mul_basis;
            n += 1;
        }

        return match negative {
            true => acc.negate(),
            _    =>  acc
        };
    }

}
