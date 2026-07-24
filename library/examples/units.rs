use qitech_framework::uom::Length;
use qitech_framework::uom::length::meter;
use qitech_framework::uom::length::millimeter;

pub fn main() {
    let x = uom_as_f64(Length::new::<millimeter>(1.0));
    println!("x: {x}");
}

fn uom_as_f64<T>(value: T) -> f64 {
    use std::mem::transmute_copy;
    const {
        assert_f64_repr::<T>();
    }
    unsafe { transmute_copy::<T, f64>(&value) }
}

const fn assert_f64_repr<T>() {
    use std::mem::align_of;
    use std::mem::size_of;
    assert!(size_of::<T>() == size_of::<f64>());
    assert!(align_of::<T>() == align_of::<f64>());
}
