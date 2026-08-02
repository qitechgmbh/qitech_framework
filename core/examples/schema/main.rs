use qitech_framework_core::schema::MachineSchema;

pub fn main() {
    let data = include_str!("mock.yaml");
    let mock_schema = MachineSchema::parse_str(data);
    println!("schema: {mock_schema:#?}");
}
