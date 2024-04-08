use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    /// Listen address
    pub listen: SocketAddr,
    /// Username of administrator.
    pub admin: String,
    /// Path to database.
    pub storage: PathBuf,
    /// Private keys.
    pub key: ConfigKey,
}

#[derive(Deserialize)]
pub struct ConfigKey {
    /// Opaque signature.
    pub opaque: String,
    /// Invitation key, used to sign generated user invitation.
    pub invitation: String,
    /// Session key, used to sign session id.
    pub session: String,
}

impl Config {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let localhost = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let default_listen = SocketAddr::new(localhost, 8080);
        let default_admin = "root";

        let mut config = Figment::new();
        if let Some(path) = path {
            let provider = Toml::file_exact(path).profile("default");
            config = config.merge(provider);
        }
        config = config
            .merge(Env::raw().split('_'))
            .join(Serialized::default("listen", default_listen))
            .join(Serialized::default("admin", default_admin));
        config
            .extract()
            .map_err(|err| anyhow!("failed to load configuration, {}", err.kind))
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv6Addr};

    use super::*;

    use claym::*;
    use figment::Jail;

    #[test]
    fn load_configuration_from_environment_variables() {
        Jail::expect_with(|jail| {
            jail.set_env("LISTEN", "[::1]:6789");
            jail.set_env("ADMIN", "xyz");
            jail.set_env("STORAGE", "/tmp/storage.sqlite");
            jail.set_env("KEY_OPAQUE", "opaque-signature");
            jail.set_env("KEY_INVITATION", "invitation-private-key");
            jail.set_env("KEY_SESSION", "session-signing-key");

            let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 6789);

            let config = assert_ok!(Config::load(None));
            assert_eq!(config.listen, addr);
            assert_eq!(config.admin, "xyz");
            assert_eq!(config.storage, Path::new("/tmp/storage.sqlite"));
            assert_eq!(config.key.opaque, "opaque-signature");
            assert_eq!(config.key.invitation, "invitation-private-key");
            assert_eq!(config.key.session, "session-signing-key");

            Ok(())
        });
    }

    #[test]
    fn load_configuration_from_configuration_file() {
        Jail::expect_with(|jail| {
            assert_ok!(jail.create_file(
                "config.toml",
                r#"
                listen = "[::1]:6789"
                admin = "xyz"
                storage = "/tmp/storage.sqlite"

                [key]
                opaque = "opaque-signature"
                invitation = "invitation-private-key"
                session = "session-signing-key"
                "#,
            ));

            let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 6789);

            let config_file = Path::new("config.toml");
            let config = assert_ok!(Config::load(Some(config_file)));
            assert_eq!(config.listen, addr);
            assert_eq!(config.admin, "xyz");
            assert_eq!(config.storage, Path::new("/tmp/storage.sqlite"));
            assert_eq!(config.key.opaque, "opaque-signature");
            assert_eq!(config.key.invitation, "invitation-private-key");
            assert_eq!(config.key.session, "session-signing-key");

            Ok(())
        });
    }
}
