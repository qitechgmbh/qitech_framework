use std::fmt::{Debug, Display};

#[derive(Debug)]
pub struct BoundsError<T> {
    received: T,
    min: Option<T>,
    max: Option<T>,
}

impl<T> Display for BoundsError<T>
where
    T: Debug + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.min, self.max) {
            (Some(min), Some(max)) => write!(
                f,
                "value {:?} is outside bounds [{:?}, {:?}]",
                self.received, min, max
            ),
            (Some(min), None) => write!(
                f,
                "value {:?} is below minimum {:?}",
                self.received, min
            ),
            (None, Some(max)) => write!(
                f,
                "value {:?} exceeds maximum {:?}",
                self.received, max
            ),
            (None, None) => write!(
                f,
                "value {:?} failed bounds validation",
                self.received
            ),
        }
    }
}

impl<T> std::error::Error for BoundsError<T>
where
    T: Debug + Display,
{}