use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

type Timestamp = DateTime<Utc>;

#[allow(dead_code)]
pub mod logs {
    use super::*;

    pub const TABLE_NAME: &str = "logs";
    pub const COLUMNS: &str = "timestamp, level, origin, message, attributes";

    pub const TIMESTAMP: &str = "timestamp";
    pub const LEVEL: &str = "level";
    pub const ORIGIN: &str = "origin";
    pub const MESSAGE: &str = "message";
    pub const ATTRIBUTES: &str = "attributes";

    pub type Timestamp = super::Timestamp;
    pub type Level = i8;
    pub type Origin = u64;
    pub type Message = String;
    pub type Attributes = Vec<(String, String)>;

    #[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
    pub struct Row {
        #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
        pub timestamp: Timestamp,
        pub level: Level,
        pub origin: Origin,
        pub message: Message,
        pub attributes: Attributes,
    }
}

#[allow(dead_code)]
pub mod events {
    use super::*;

    pub const TABLE_NAME: &str = "events";
    pub const COLUMNS: &str = "timestamp, origin, name, value";

    pub const TIMESTAMP: &str = "timestamp";
    pub const ORIGIN: &str = "origin";
    pub const NAME: &str = "name";
    pub const VALUE: &str = "value";

    pub type Timestamp = super::Timestamp;
    pub type Origin = u64;
    pub type Name = String;
    pub type Value = String;

    #[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
    pub struct Row {
        #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
        pub timestamp: Timestamp,
        pub origin: Origin,
        pub name: Name,
        pub value: Value,
    }
}

#[allow(dead_code)]
pub mod machine_activity {
    use super::*;

    pub const TABLE_NAME: &str = "machine_activity";
    pub const COLUMNS: &str = "identity, updated_at";

    pub const IDENTITY: &str = "identity";
    pub const UPDATED_AT: &str = "updated_at";

    pub type Identity = u64;
    pub type Timestamp = super::Timestamp;

    #[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
    pub struct Row {
        pub identity: u64,

        #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
        pub updated_at: Timestamp,
    }
}

#[allow(dead_code)]
pub mod machine_config_mutations {
    use super::*;

    pub const TABLE_NAME: &str = "machine_config_mutations";
    pub const COLUMNS: &str = "timestamp, identity, name, value_type, value_enum, 
        value_string, value_int, value_float, value_bool, origin, result";

    pub const TIMESTAMP: &str = "timestamp";
    pub const IDENTITY: &str = "identity";
    pub const NAME: &str = "name";
    pub const VALUE_TYPE: &str = "value_type";
    pub const VALUE_ENUM: &str = "value_enum";
    pub const VALUE_STRING: &str = "value_string";
    pub const VALUE_INT: &str = "value_int";
    pub const VALUE_FLOAT: &str = "value_float";
    pub const VALUE_BOOL: &str = "value_bool";
    pub const ORIGIN: &str = "origin";
    pub const RESULT: &str = "result";

    pub type Timestamp = DateTime<Utc>;
    pub type Identity = u64;
    pub type Name = String;
    pub type ValueType = i8;
    pub type ValueEnum = String;
    pub type ValueString = Option<String>;
    pub type ValueInt = Option<i64>;
    pub type ValueFloat = Option<f64>;
    pub type ValueBool = Option<bool>;
    pub type Origin = u64;
    pub type Result = i8;

    #[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
    pub struct Row {
        #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
        pub timestamp: Timestamp,

        pub machine: Identity,
        pub path: Name,

        pub value_type: ValueType,
        pub value_enum: ValueEnum,
        pub value_string: ValueString,
        pub value_int: ValueInt,
        pub value_float: ValueFloat,
        pub value_bool: ValueBool,

        pub origin: Origin,
        pub result: Result,
    }
}

#[allow(dead_code)]
pub mod machine_state_mutations {
    use super::*;

    pub const TABLE_NAME: &str = "machine_state_mutations";
    pub const COLUMNS: &str = "timestamp, identity, name, value_type, value_enum, value_string, 
        value_int, value_float, value_bool";

    pub const TIMESTAMP: &str = "timestamp";
    pub const IDENTITY: &str = "identity";
    pub const NAME: &str = "name";
    pub const VALUE_TYPE: &str = "value_type";
    pub const VALUE_ENUM: &str = "value_enum";
    pub const VALUE_STRING: &str = "value_string";
    pub const VALUE_INT: &str = "value_int";
    pub const VALUE_FLOAT: &str = "value_float";
    pub const VALUE_BOOL: &str = "value_bool";

    pub type Timestamp = DateTime<Utc>;
    pub type Identity = u64;
    pub type Name = String;
    pub type ValueType = i8;
    pub type ValueEnum = String;
    pub type ValueString = Option<String>;
    pub type ValueInt = Option<i64>;
    pub type ValueFloat = Option<f64>;
    pub type ValueBool = Option<bool>;

    #[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
    pub struct Row {
        #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
        pub timestamp: Timestamp,

        pub identity: Identity,
        pub path: Name,

        pub value_type: ValueType,
        pub value_enum: ValueEnum,
        pub value_string: ValueString,
        pub value_int: ValueInt,
        pub value_float: ValueFloat,
        pub value_bool: ValueBool,
    }
}

#[allow(dead_code)]
pub mod machine_measurements {
    use super::*;

    pub const TABLE_NAME: &str = "machine_measurements";
    pub const COLUMNS: &str = "timestamp, identity, name, value";

    pub const TIMESTAMP: &str = "timestamp";
    pub const IDENTITY: &str = "identity";
    pub const NAME: &str = "name";
    pub const VALUE: &str = "value";

    pub type Timestamp = DateTime<Utc>;
    pub type Identity = u64;
    pub type Name = String;
    pub type Value = Option<f64>;

    #[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
    pub struct Row {
        #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
        pub timestamp: Timestamp,

        pub identity: Identity,
        pub name: Name,
        pub value: Value,
    }
}
