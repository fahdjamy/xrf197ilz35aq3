use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Environment {
    Dev,
    Live,
    Staging,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Dev => "dev",
            Environment::Live => "live",
            Environment::Staging => "stg",
            Environment::Production => "prod",
        }
    }

    pub fn is_local(&self) -> bool {
        *self == Environment::Dev
    }

    pub fn is_not_local(&self) -> bool {
        !self.is_local()
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(env: String) -> Result<Self, Self::Error> {
        match env.to_lowercase().as_str() {
            "live" => Ok(Environment::Live),
            "stg" => Ok(Environment::Staging),
            "prod" => Ok(Environment::Production),
            "dev" | "local" => Ok(Environment::Dev),
            _ => Err(format!("Unknown environment: {}", env)),
        }
    }
}
