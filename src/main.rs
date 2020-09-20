
use fixed_point::*;


fn main() {

    let mut time = Fixed::ZERO;
    let timestep = Fixed::from_raw(1);

    loop{
        time = time + timestep;
        if time > Fixed::TAU {
            break;
        }

        let sine = time.to_f32().sin();
        let sine_m = time.sin();
        let err = sine - sine_m.to_f32();
        
        if err.abs() > 0.00001{
            println!("{} - {} = {}", sine, sine_m.to_f32(), err);
        }
    }

    for i in 1<<12..1<<30{
        let f = Fixed::from_raw(i);

        let inv_f = f.inverse();
        let inv = 1.0 / f.to_f32();

        let err = inv_f.to_f32() - inv;

        if err.abs() > 0.001{
            println!("{} - {} = {}", inv, inv_f.to_f32(), err);
        }
    }
    
}
