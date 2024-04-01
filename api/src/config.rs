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

#[derive(Debug, Deserialize)]
pub struct Config {
    pub listen_addr: SocketAddr,
    pub opaque_signature: String,
    pub admin_user: String,
    pub storage: PathBuf,
    pub auth_token: String,
}

impl Config {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let localhost = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let default_listen_addr = SocketAddr::new(localhost, 8080);
        let default_admin_uer = "root";

        let mut config = Figment::new();
        if let Some(path) = path {
            let provider = Toml::file_exact(path).profile("default");
            config = config.merge(provider);
        }
        config = config
            .merge(Env::raw())
            .join(Serialized::default("listen_addr", default_listen_addr))
            .join(Serialized::default("admin_user", default_admin_uer));
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
            jail.set_env("LISTEN_ADDR", "[::1]:6789");
            jail.set_env("OPAQUE_SIGNATURE", "abcdefgh");
            jail.set_env("ADMIN_USER", "xyz");
            jail.set_env("STORAGE", "/tmp/storage.sqlite");
            jail.set_env("AUTH_TOKEN", "123abc456");

            let config = assert_ok!(Config::load(None));
            assert_eq!(
                config.listen_addr,
                SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 6789)
            );
            assert_eq!(config.opaque_signature, "abcdefgh");
            assert_eq!(config.admin_user, "xyz");
            assert_eq!(config.auth_token, "123abc456");

            Ok(())
        });
    }

    #[test]
    fn load_configuration_from_configuration_file() {
        Jail::expect_with(|jail| {
            assert_ok!(jail.create_file(
                "config.toml",
                r#"
                listen_addr = "[::1]:6789"
                opaque_signature = "abcdefgh"
                admin_user = "xyz"
                storage = "/tmp/storage.sqlite"
                auth_token = "123abc456"
                "#,
            ));

            let config_file = Path::new("config.toml");
            let config = assert_ok!(Config::load(Some(config_file)));
            assert_eq!(
                config.listen_addr,
                SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 6789)
            );
            assert_eq!(config.opaque_signature, "abcdefgh");
            assert_eq!(config.admin_user, "xyz");
            assert_eq!(config.auth_token, "123abc456");

            Ok(())
        });
    }
}
