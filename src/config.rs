use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct GlobalConfig {}

impl GlobalConfig {
    async fn new() -> Self {
        let env = Environment::from_env();
        let app_name = env.config_details.app_name;
        let email = env.config_details.email;
        let client = reqwest::Client::new();
        let payload = serde_json::json! {{
            "action": "GetItem",
            "app": app_name,
            "email": email,
        }};
        let _response = client
            .post(env.config_details.url)
            .header("x-api-key", env.config_details.api_key)
            .json(&payload)
            .send();
        Self {}
    }
}


#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Global {
    pub env: Environment,
    pub config: GlobalConfig,
}

impl Global {
    async fn new() -> Self {
        Self {
            config: GlobalConfig::new().await,
            env: Environment::from_env(),
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ConfigDetails {
    pub url: String,
    pub api_key: String,
    pub app_name: String,
    pub environment: String,
    pub email: String,
}

#[derive(Eq, PartialEq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Environment {
    pub host: String,
    pub port: u16,
    pub env: String,
    pub config_details: ConfigDetails,
}


impl Environment {
    pub(crate) fn from_env() -> Self {
        let host: String = match std::env::var("HOST") {
            Ok(v) => v,
            _ => "0.0.0.0".into(),
        };
        let port: u16 = match std::env::var("PORT") {
            Ok(v) => v.parse::<u16>().expect("Invalid PORT number was provided"),
            _ => 80,
        };

        let env: String = match std::env::var("ENV") {
            Ok(v) => v,
            _ => "dev".to_owned(),
        };

        let config_url: String = std::env::var("CONFIG_CONCORD_URL")
            .expect("Provide Url for config server; export CONFIG_CONCORD_URL");


        let config_api_key: String = std::env::var("CONFIG_CONCORD_API_KEY")
            .expect("Provide Api Key for config server; export CONFIG_CONCORD_API_KEY");

        let config_email: String = std::env::var("CONFIG_CONCORD_EMAIL")
            .expect("Provide Api Key for config server; export CONFIG_CONCORD_EMAIL");

        let mut config_app_name: String = std::env::var("CONFIG_CONCORD_APP_NAME")
            .expect("Provide Api Key for config server; export CONFIG_CONCORD_APP_NAME");

        let config_env = env.clone();

        if !config_app_name.ends_with(config_env.as_str()) {
            config_app_name = format!("{config_app_name}-{config_env}");
        }

        let config_details = ConfigDetails {
            url: config_url,
            api_key: config_api_key,
            email: config_email,
            environment: config_env,
            app_name: config_app_name,
        };

        Self {
            host,
            port,
            env,
            config_details,
        }
    }

    pub(crate) fn host_port(&self) -> (String, u16) {
        (self.host.clone(), self.port)
    }

    pub(crate) fn is_dev(&self) -> bool {
        self.env.to_ascii_lowercase().contains("dev")
    }
}

/// ```rust
/// use std::sync::Arc;
/// use crate::config::*;
///
/// if let Some(global) = global! () {
///     // use global config
/// } else {
///     // failed to get global config
/// }
/// ```
macro_rules! global {
    () => {{
        let g = Arc::clone(&GLOBAL_MUTEX);
        let lock = g.lock();
        match lock {
            Ok(obj) => Some(obj.clone()),
            Err(err) => {
                println!("ERROR: Failed to acquire global resource lock '{:?}'", err);
                None
            }
        }
    }};
}

lazy_static! {
    pub static ref GLOBAL_MUTEX: Arc<Mutex<Option<Global>>> = Arc::new(Mutex::new(None));
}