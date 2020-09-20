
#![no_std]
use core::ops::{Add, Sub, Mul, Div, Rem};

const SCALE_BITS: i32 = 12;
const SCALE_INV: i32 = 32 - SCALE_BITS;

#[derive(Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
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

    pub fn sin(self: Fixed) -> Fixed{
        const PI  :Fixed = Fixed::from_raw(3294199); //Fixed::from_decimal(3_14159265, 8);
        const HPI :Fixed = Fixed::from_raw(3294199 >> 1);
        let flip = self > PI;
        
        // The first half is more accurate than the second
        let t = match flip {
            true => self - PI,
            _ => self
        };

        // and the first quarter is even more accurate
        let t = match t > HPI {
            true => HPI - (t - HPI),
            _ => t
        };
    
        // 1/(2*3), 1/(4*5)...
        const I23   :Fixed = Fixed::from_raw(174763);
        const I45   :Fixed = Fixed::from_raw(52429);
        const I67   :Fixed = Fixed::from_raw(24966);
        const I89   :Fixed = Fixed::from_raw(14564);
    
    
        let t2 = t * t;
        let t3 = t * t2;
    
        let t2_45   = t2 * I45;   // t^2 / 4*5
        let t2_67   = t2 * I67;   // t^2 / 6*7
        let t2_89   = t2 * I89;   // etc
        
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
}