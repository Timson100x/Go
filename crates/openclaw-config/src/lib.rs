//! YAML configuration loader with `${ENV_VAR}` interpolation.
//!
//! # Example
//! ```no_run
//! use openclaw_config::load;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct MyConfig {
//!     endpoint: String,
//! }
//!
//! let cfg: MyConfig = load("config/grpc_streamer.yaml").unwrap();
//! ```

use std::{fs, path::Path};

use anyhow::{Context, Result};
use regex::Regex;
use serde::de::DeserializeOwned;

/// Load a YAML file at `path`, perform `${VAR}` → env interpolation, then
/// deserialise into `T`.
pub fn load<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path)
        .with_context(|| format!("cannot read config file {}", path.display()))?;
    let interpolated = interpolate_env(&raw)?;
    serde_yaml::from_str(&interpolated)
        .with_context(|| format!("cannot parse YAML config {}", path.display()))
}

/// Replace every `${VAR_NAME}` occurrence in `text` with the corresponding
/// environment variable value.  Missing variables produce an error.
fn interpolate_env(text: &str) -> Result<String> {
    let re = Regex::new(r"\$\{([^}]+)\}").expect("static regex is valid");
    let mut error: Option<anyhow::Error> = None;
    let result = re.replace_all(text, |caps: &regex::Captures<'_>| {
        let var = &caps[1];
        match std::env::var(var) {
            Ok(val) => val,
            Err(_) => {
                if error.is_none() {
                    error = Some(anyhow::anyhow!("environment variable `{}` is not set", var));
                }
                String::new()
            }
        }
    });
    match error {
        Some(e) => Err(e),
        None => Ok(result.into_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_env_var() {
        std::env::set_var("TEST_ADDR", "localhost:9000");
        let out = interpolate_env("endpoint: ${TEST_ADDR}").unwrap();
        assert_eq!(out, "endpoint: localhost:9000");
    }

    #[test]
    fn missing_var_returns_error() {
        std::env::remove_var("DEFINITELY_NOT_SET_XYZ");
        let result = interpolate_env("val: ${DEFINITELY_NOT_SET_XYZ}");
        assert!(result.is_err());
    }
}
