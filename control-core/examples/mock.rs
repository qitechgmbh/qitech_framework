use control_core::schema;

pub fn main() {
    let schema = schema::parse_latest(include_str!("mock.rs")).unwrap();
    println!("schema");
}