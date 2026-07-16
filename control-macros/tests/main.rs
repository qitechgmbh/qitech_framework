use chrono::Utc;
use control_hub_macros::row;

// #[derive(Machine))]
// #[machine(name = "laser_v1")]

#[test]
fn some() {
    row! {
        logs {
            timestamp,
            level,
            origin,
            attributes,
            message,
        }
    }

    let x = LogsRow {
        timestamp: Utc::now(),
        level: 0,
        origin: 0,
        message: "Hello World".to_string(),
        attributes: Default::default(),
    };

    println!("{x:?}");
}