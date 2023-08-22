use crate::error::Error;

pub trait TokenValidator {
    fn validate(&self, token: &str) -> Result<(), Error>;
}
