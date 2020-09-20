
use fixed_point::*;


fn main() {

    let bad_pi = Fixed::from_decimal(3_14159265, 8);
    let bad_tau:Fixed = bad_pi * 2;
   
    let mut time = Fixed::ZERO;
    let timestep = Fixed::from_decimal(0_0001, 4);

    loop{
        time = time + timestep;
        if time > bad_tau {
            return;
        }

        let sine = time.to_f32().sin();
        let sine_m = time.sin();
        let err = sine - sine_m.to_f32();
        
        if err.abs() > 0.00001{
            println!("{} - {} = {}", sine, sine_m.to_f32(), err);
        }
    }
}
