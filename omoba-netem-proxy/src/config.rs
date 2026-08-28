use std::{collections::BTreeSet, env, net::SocketAddr, path::PathBuf, time::Duration};

use serde::Serialize;

use crate::{delay::auto_seed, profile::DelayProfile, NetemError, Result};

pub const DEFAULT_MAX_DATAGRAMS: usize = 4096;
pub const DEFAULT_MAX_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DelayMode {
    OrderedDelay,
    NaturalReorder,
}

impl DelayMode {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "ordered-delay" => Ok(Self::OrderedDelay),
            "natural-reorder" => Ok(Self::NaturalReorder),
            _ => Err(NetemError::Config(format!("invalid mode {value}"))),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RouteConfig {
    pub team_id: u32,
    pub client_bind: SocketAddr,
    pub upstream_bind: SocketAddr,
    pub initial_profile: DelayProfile,
}

#[derive(Clone, Debug)]
pub struct ProxyConfig {
    pub server_addr: SocketAddr,
    pub control_bind: SocketAddr,
    pub routes: [RouteConfig; 2],
    pub mode: DelayMode,
    pub seed: u64,
    pub evidence_dir: PathBuf,
    pub max_datagrams: usize,
    pub max_bytes: usize,
    pub watchdog: Duration,
}

#[derive(Serialize)]
pub struct SanitizedConfig<'a> {
    pub server_addr: SocketAddr,
    pub control_bind: SocketAddr,
    pub routes: &'a [RouteConfig; 2],
    pub mode: DelayMode,
    pub seed: u64,
    pub max_datagrams: usize,
    pub max_bytes: usize,
    pub watchdog_ms: u128,
}

impl ProxyConfig {
    pub fn from_env_args() -> Result<Self> {
        Self::parse(env::args().skip(1))
    }

    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut values = args.into_iter().collect::<Vec<_>>().into_iter();
        let mut server_addr = None;
        let mut control_bind = None;
        let mut client = [None, None];
        let mut upstream = [None, None];
        let mut profiles = ["uniform-20-100".to_string(), "uniform-20-100".to_string()];
        let mut customs: [Option<PathBuf>; 2] = [None, None];
        let mut mode = DelayMode::OrderedDelay;
        let mut seed = None;
        let mut evidence_dir = None;
        let mut max_datagrams = DEFAULT_MAX_DATAGRAMS;
        let mut max_bytes = DEFAULT_MAX_BYTES;
        let mut watchdog_ms = 5000_u64;
        while let Some(flag) = values.next() {
            let value = |values: &mut std::vec::IntoIter<String>| {
                values
                    .next()
                    .ok_or_else(|| NetemError::Config(format!("missing value for {flag}")))
            };
            match flag.as_str() {
                "--server" => server_addr = Some(parse_addr(&value(&mut values)?, &flag)?),
                "--control-bind" => control_bind = Some(parse_addr(&value(&mut values)?, &flag)?),
                "--team1-client-bind" => client[0] = Some(parse_addr(&value(&mut values)?, &flag)?),
                "--team2-client-bind" => client[1] = Some(parse_addr(&value(&mut values)?, &flag)?),
                "--team1-upstream-bind" => {
                    upstream[0] = Some(parse_addr(&value(&mut values)?, &flag)?)
                }
                "--team2-upstream-bind" => {
                    upstream[1] = Some(parse_addr(&value(&mut values)?, &flag)?)
                }
                "--team1-profile" => profiles[0] = value(&mut values)?,
                "--team2-profile" => profiles[1] = value(&mut values)?,
                "--team1-custom" => customs[0] = Some(PathBuf::from(value(&mut values)?)),
                "--team2-custom" => customs[1] = Some(PathBuf::from(value(&mut values)?)),
                "--mode" => mode = DelayMode::parse(&value(&mut values)?)?,
                "--seed" => seed = Some(parse(&value(&mut values)?, &flag)?),
                "--evidence-dir" => evidence_dir = Some(PathBuf::from(value(&mut values)?)),
                "--max-datagrams" => max_datagrams = parse(&value(&mut values)?, &flag)?,
                "--max-bytes" => max_bytes = parse(&value(&mut values)?, &flag)?,
                "--watchdog-ms" => watchdog_ms = parse(&value(&mut values)?, &flag)?,
                _ => return Err(NetemError::Config(format!("unknown argument {flag}"))),
            }
        }
        let required = |value: Option<SocketAddr>, name: &str| {
            value.ok_or_else(|| NetemError::Config(format!("{name} is required")))
        };
        let server_addr = required(server_addr, "--server")?;
        let control_bind = required(control_bind, "--control-bind")?;
        let client = [
            required(client[0], "--team1-client-bind")?,
            required(client[1], "--team2-client-bind")?,
        ];
        let upstream = [
            required(upstream[0], "--team1-upstream-bind")?,
            required(upstream[1], "--team2-upstream-bind")?,
        ];
        for address in [control_bind, client[0], client[1], upstream[0], upstream[1]] {
            if !address.ip().is_loopback() {
                return Err(NetemError::Config(format!(
                    "bind must be loopback: {address}"
                )));
            }
        }
        let ports: BTreeSet<_> = [control_bind, client[0], client[1], upstream[0], upstream[1]]
            .into_iter()
            .map(|address| address.port())
            .collect();
        if ports.len() != 5 {
            return Err(NetemError::Config("proxy bind ports must be unique".into()));
        }
        if max_datagrams == 0 || max_bytes == 0 || watchdog_ms == 0 {
            return Err(NetemError::Config(
                "queue budgets and watchdog must be positive".into(),
            ));
        }
        let profile = |index: usize| match (&customs[index], profiles[index].as_str()) {
            (Some(path), "custom-20-bin") => DelayProfile::load_custom(path),
            (None, name) => DelayProfile::named(name),
            (Some(_), _) => Err(NetemError::Config(
                "custom path requires custom-20-bin profile".into(),
            )),
        };
        Ok(Self {
            server_addr,
            control_bind,
            routes: [
                RouteConfig {
                    team_id: 1,
                    client_bind: client[0],
                    upstream_bind: upstream[0],
                    initial_profile: profile(0)?,
                },
                RouteConfig {
                    team_id: 2,
                    client_bind: client[1],
                    upstream_bind: upstream[1],
                    initial_profile: profile(1)?,
                },
            ],
            mode,
            seed: seed.unwrap_or_else(auto_seed),
            evidence_dir: evidence_dir
                .ok_or_else(|| NetemError::Config("--evidence-dir is required".into()))?,
            max_datagrams,
            max_bytes,
            watchdog: Duration::from_millis(watchdog_ms),
        })
    }

    pub fn sanitized(&self) -> SanitizedConfig<'_> {
        SanitizedConfig {
            server_addr: self.server_addr,
            control_bind: self.control_bind,
            routes: &self.routes,
            mode: self.mode,
            seed: self.seed,
            max_datagrams: self.max_datagrams,
            max_bytes: self.max_bytes,
            watchdog_ms: self.watchdog.as_millis(),
        }
    }
}

fn parse_addr(value: &str, flag: &str) -> Result<SocketAddr> {
    value
        .parse()
        .map_err(|_| NetemError::Config(format!("invalid socket address for {flag}")))
}

fn parse<T: std::str::FromStr>(value: &str, flag: &str) -> Result<T> {
    value
        .parse()
        .map_err(|_| NetemError::Config(format!("invalid integer for {flag}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args() -> Vec<String> {
        vec![
            "--server",
            "127.0.0.1:50061",
            "--control-bind",
            "127.0.0.1:63200",
            "--team1-client-bind",
            "127.0.0.1:63001",
            "--team2-client-bind",
            "127.0.0.1:63002",
            "--team1-upstream-bind",
            "127.0.0.1:63101",
            "--team2-upstream-bind",
            "127.0.0.1:63102",
            "--seed",
            "1",
            "--evidence-dir",
            "evidence",
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    }
    #[test]
    fn parses_safe_loopback_config() {
        let value = ProxyConfig::parse(args()).unwrap();
        assert_eq!(value.seed, 1);
        assert_eq!(value.routes[0].team_id, 1)
    }
    #[test]
    fn rejects_unknown_remote_and_duplicate() {
        let mut unknown = args();
        unknown.extend(["--wat".into(), "1".into()]);
        assert!(ProxyConfig::parse(unknown).is_err());
        let mut remote = args();
        let index = remote.iter().position(|v| v == "127.0.0.1:63001").unwrap();
        remote[index] = "0.0.0.0:63001".into();
        assert!(ProxyConfig::parse(remote).is_err());
        let mut duplicate = args();
        let index = duplicate
            .iter()
            .position(|v| v == "127.0.0.1:63002")
            .unwrap();
        duplicate[index] = "127.0.0.1:63001".into();
        assert!(ProxyConfig::parse(duplicate).is_err())
    }
}
