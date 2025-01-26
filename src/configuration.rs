use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

#[derive(Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: String,
}

impl ApplicationSettings {
    pub fn application_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }

    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // get the path to the config directory
    let base_path = std::env::current_dir().expect("unable to determine current directory");
    let configuration_directory = base_path.join("configuration");

    // determine the current environment
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or("LOCAL".to_string())
        .try_into()
        .expect("unable to determine app environment");

    let env_specific_settings_file = format!("{}.yml", environment.as_str());

    let settings = config::Config::builder()
        .add_source(config::File::from(configuration_directory.join("base.yml")))
        .add_source(config::File::from(
            configuration_directory.join(env_specific_settings_file),
        ))
        .build()?;

    settings.try_deserialize::<Settings>()
}

enum Environment {
    LOCAL,
    PRODUCTION,
}

impl Environment {
    pub fn as_str(&self) -> &str {
        match self {
            Self::LOCAL => "local",
            Self::PRODUCTION => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::LOCAL),
            "production" => Ok(Self::PRODUCTION),
            _ => Err(format!(
                "{value} is not a supported environment. \
                 Use either `local` or `production`."
            )),
        }
    }
}
