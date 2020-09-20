
#![no_std]
use core::ops::{Add, Sub, Mul, Div, Rem};

const SCALE_BITS: i32 = 12;
const SCALE_INV: i32 = 32 - SCALE_BITS;

const FRACTION_MASK: i32 = (1 << SCALE_INV) - 1;

#[derive(Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Debug)]
pub struct Fixed{
    value: i32
}

// impl Ord for Fixed{
//     fn cmp(self, o: Fixed) -> Ordering{
//         self.value.cmp(o.value);
//     }
// }

impl Rem for Fixed{
    type Output = Self;
    fn rem(self, modulus: Self) -> Self::Output {
        Fixed{value: self.value % modulus.value}
    }
}

impl Add<Fixed> for Fixed {
    type Output = Fixed;
    fn add(self, o: Self)-> Fixed{
        Fixed{value: self.value + o.value}
    }
}

impl Sub<Fixed> for Fixed {
    type Output = Fixed;
    fn sub(self, o: Self)-> Fixed{
        Fixed{value: self.value - o.value}
    }
}

impl Mul<Fixed> for Fixed {
    type Output = Fixed;
    fn mul(self, o: Self)-> Fixed{
        let intermediate:i64 = self.value as i64 * o.value as i64;
        Fixed{value: (intermediate >> SCALE_INV) as i32}
    }
}

impl Div<Fixed> for Fixed{
    type Output = Fixed;
    fn div(self, o: Self) -> Fixed{
        let dividend = (self.value as i64) << 32u16;
        let intermediate = dividend / o.value as i64;

        Fixed{value: (intermediate >> SCALE_BITS) as i32}
    }   
}

/* i32 variants */
impl Add<i32> for Fixed {
    type Output = Fixed;
    fn add(self, o: i32)-> Fixed{
        Fixed{value: self.value + (o << SCALE_INV)}
    }
}

impl Sub<i32> for Fixed {
    type Output = Fixed;
    fn sub(self, o: i32)-> Fixed{
        Fixed{value: self.value - (o << SCALE_INV)}
    }
}

impl Mul<i32> for Fixed {
    type Output = Fixed;
    fn mul(self, o: i32)-> Fixed{
        Fixed{value: self.value * o}
    }
}

impl Div<i32> for Fixed{
    type Output = Fixed;
    fn div(self, o: i32) -> Fixed{
        Fixed{value: self.value / o}
    }
}

/* i32 LHS variants */
impl Add<Fixed> for i32 {
    type Output = Fixed;
    fn add(self, o: Fixed)-> Fixed{
        Fixed{value: (self << SCALE_INV) + o.value}
    }
}

impl Sub<Fixed> for i32 {
    type Output = Fixed;
    fn sub(self, o: Fixed)-> Fixed{
        Fixed{value: (self << SCALE_INV) - o.value}
    }
}

impl Mul<Fixed> for i32 {
    type Output = Fixed;
    fn mul(self, o: Fixed)-> Fixed{
        Fixed{value: self * o.value}
    }
}

impl Div<Fixed> for i32{
    type Output = Fixed;
    fn div(self, o: Fixed) -> Fixed{
        let intermediate = ((self as i64) << 32u16) / o.value as i64;
        Fixed{value: (intermediate << (SCALE_INV - (32 - SCALE_INV))) as i32}
    }
}

impl Fixed {
    pub fn to_f32(&self) -> f32{
        self.value as f32 / ((1 << SCALE_INV) as f32)
    }

    pub fn to_i32(&self) -> i32{
        self.value >> SCALE_INV
    }

    pub fn from_i32(i: i32) -> Fixed{
        Fixed{value: i << SCALE_INV}
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

        // const CROSSOVER:Fixed = Fixed{value: 524183 };
        // if x > CROSSOVER{
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
            let taylor_pt = 1 << n;
            // 1 / taylor_pt
            let taylor_i = Fixed{value: 1 << (SCALE_INV - n)};

            // Multiplicitive basis, each term is multiplied by this to get the next term
            let mul_basis = taylor_i.negate() * (-taylor_pt + x);

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
                let next = guess * (2 - (x * guess));
                if (next.value - guess.value).abs() < 100{
                    return next;
                }
                guess = next;
            }
            panic!("failed to converge");
        }
    }

}