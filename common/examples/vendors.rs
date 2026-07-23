use qitech_framework_common::vendors;

pub fn main() {
    let vendor_name = vendors::get_name(1).unwrap();
    println!("vendor with id 1 is '{vendor_name}'");
}