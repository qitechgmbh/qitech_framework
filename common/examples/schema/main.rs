use qitech_framework_common::MachineSchema;

pub fn main() {
    let data = include_str!("mock.yaml");
    let mock_schema = MachineSchema::from_yaml_str(data);
    println!("schema: {mock_schema:#?}");
}
