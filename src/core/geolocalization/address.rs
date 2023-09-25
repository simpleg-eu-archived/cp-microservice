use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Address {
    country: String,
    region: String,
    city: String,
    street: String,
    number: String,
    additional: String,
    postal_code: String,
}

impl Default for Address {
    fn default() -> Self {
        Self {
            country: "".to_string(),
            region: "".to_string(),
            city: "".to_string(),
            street: "".to_string(),
            number: "".to_string(),
            additional: "".to_string(),
            postal_code: "".to_string(),
        }
    }
}

impl Address {
    pub fn new(
        country: String,
        region: String,
        city: String,
        street: String,
        number: String,
        additional: String,
        postal_code: String,
    ) -> Self {
        Self {
            country,
            region,
            city,
            street,
            number,
            additional,
            postal_code,
        }
    }

    pub fn country(&self) -> &str {
        &self.country
    }

    pub fn region(&self) -> &str {
        &self.region
    }

    pub fn city(&self) -> &str {
        &self.city
    }

    pub fn street(&self) -> &str {
        &self.street
    }

    pub fn number(&self) -> &str {
        &self.number
    }

    pub fn additional(&self) -> &str {
        &self.additional
    }

    pub fn postal_code(&self) -> &str {
        &self.postal_code
    }
}
