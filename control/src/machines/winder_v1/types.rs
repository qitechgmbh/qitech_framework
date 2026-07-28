use qitech_framework::ScalarValue;
use qitech_framework::machine::BoundedMeta;
use qitech_framework::machine::TypeWrapper;
use serde::Deserialize;

#[derive(Clone, PartialEq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    Standby,
    Hold,
    Pull,
    Wind,
}

impl TypeWrapper for Mode {
    type Type = Mode;
    type Input = Mode;

    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::Enum(Some(
            match value {
                Self::Standby => "standby",
                Self::Hold => "hold",
                Self::Pull => "pull",
                Self::Wind => "wind",
            }
            .to_owned(),
        ))
    }

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn deserialize_json(raw: &str) -> serde_json::Result<Self> {
        match raw {
            "standby" => Ok(Self::Standby),
            "hold" => Ok(Self::Hold),
            "pull" => Ok(Self::Pull),
            "wind" => Ok(Self::Wind),
            _ => Err(serde::de::Error::custom(format!(
                "unknown Mode variant: {raw}"
            ))),
        }
    }
}

impl BoundedMeta for Mode {
    type Bound = u64;
    fn as_bound(&self) -> Option<Self::Bound> {
        None
    }
}
