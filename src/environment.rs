use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Environment {
    Dev,
    Live,
    Test,
    Local,
    Staging,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Dev => "dev",
            Environment::Live => "live",
            Environment::Test => "live",
            Environment::Local => "local",
            Environment::Staging => "stg",
            Environment::Production => "prod",
        }
    }

    pub fn is_local(&self) -> bool {
        *self == Environment::Local || *self == Environment::Dev
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
            "dev" | "DEV" => Ok(Environment::Local),
            "live" | "LIVE" => Ok(Environment::Live),
            "local" | "LOCAL" => Ok(Environment::Local),
            "test" | "TEST" | "TESTING" => Ok(Environment::Test),
            "stg" | "STAG" | "STAGING" => Ok(Environment::Staging),
            "prod" | "PROD" | "PRODUCTION" => Ok(Environment::Production),
            _ => Err(format!("Unknown environment: {}", env)),
        }
    }
}
