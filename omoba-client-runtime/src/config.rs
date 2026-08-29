use std::{net::SocketAddr, path::PathBuf};

use crate::ClientRuntimeError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientRuntimeConfig {
    pub player_id: u32,
    pub team_id: u32,
    pub player_name: String,
    pub server_addr: SocketAddr,
    pub presentation_bind: SocketAddr,
    pub presentation_hz: u32,
    pub evidence_dir: Option<PathBuf>,
    pub test_mode: bool,
    pub protocol_version: u32,
    pub content_hash: String,
    pub scripted_move_tick: Option<u64>,
    pub scripted_hidden_target_tick: Option<u64>,
    pub screenshot_tick: Option<u64>,
    pub fault_tick: Option<u64>,
    pub rebase_probe_tick: Option<u64>,
    pub shutdown_file: Option<PathBuf>,
}

impl ClientRuntimeConfig {
    pub fn from_env_args() -> Result<Self, ClientRuntimeError> {
        Self::parse(std::env::args().skip(1))
    }

    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, ClientRuntimeError> {
        let mut player_id = None;
        let mut team_id = None;
        let mut player_name = None;
        let mut server_addr = None;
        let mut presentation_bind = None;
        let mut presentation_hz = 60;
        let mut evidence_dir = None;
        let mut test_mode = false;
        let mut protocol_version = 2;
        let mut content_hash = String::new();
        let mut scripted_move_tick = None;
        let mut scripted_hidden_target_tick = None;
        let mut screenshot_tick = None;
        let mut fault_tick = None;
        let mut rebase_probe_tick = None;
        let mut shutdown_file = None;
        let mut args = args.into_iter();
        while let Some(flag) = args.next() {
            let value = |args: &mut dyn Iterator<Item = String>| {
                args.next()
                    .ok_or_else(|| ClientRuntimeError::Config(format!("missing value for {flag}")))
            };
            match flag.as_str() {
                "--player-id" => player_id = Some(parse_u32(&value(&mut args)?, &flag)?),
                "--team" | "--team-id" => team_id = Some(parse_u32(&value(&mut args)?, &flag)?),
                "--player-name" => player_name = Some(value(&mut args)?),
                "--server" => server_addr = Some(parse_addr(&value(&mut args)?, &flag)?),
                "--presentation-bind" => {
                    presentation_bind = Some(parse_addr(&value(&mut args)?, &flag)?)
                }
                "--presentation-hz" => presentation_hz = parse_u32(&value(&mut args)?, &flag)?,
                "--evidence-dir" => evidence_dir = Some(PathBuf::from(value(&mut args)?)),
                "--test-mode" => test_mode = true,
                "--protocol-version" => protocol_version = parse_u32(&value(&mut args)?, &flag)?,
                "--content-hash" => content_hash = value(&mut args)?,
                "--scripted-move-tick" => {
                    scripted_move_tick = Some(value(&mut args)?.parse().map_err(|_| {
                        ClientRuntimeError::Config("invalid scripted move tick".into())
                    })?)
                }
                "--scripted-hidden-target-tick" => {
                    scripted_hidden_target_tick = Some(value(&mut args)?.parse().map_err(|_| {
                        ClientRuntimeError::Config("invalid scripted hidden target tick".into())
                    })?)
                }
                "--screenshot-tick" => {
                    screenshot_tick = Some(value(&mut args)?.parse().map_err(|_| {
                        ClientRuntimeError::Config("invalid screenshot tick".into())
                    })?)
                }
                "--fault-tick" => {
                    fault_tick = Some(
                        value(&mut args)?
                            .parse()
                            .map_err(|_| ClientRuntimeError::Config("invalid fault tick".into()))?,
                    )
                }
                "--rebase-probe-tick" => {
                    rebase_probe_tick = Some(value(&mut args)?.parse().map_err(|_| {
                        ClientRuntimeError::Config("invalid rebase probe tick".into())
                    })?)
                }
                "--shutdown-file" => shutdown_file = Some(PathBuf::from(value(&mut args)?)),
                _ => {
                    return Err(ClientRuntimeError::Config(format!(
                        "unknown argument {flag}"
                    )))
                }
            }
        }
        let team_id =
            team_id.ok_or_else(|| ClientRuntimeError::Config("--team is required".into()))?;
        if !matches!(team_id, 1 | 2) {
            return Err(ClientRuntimeError::Config("--team must be 1 or 2".into()));
        }
        let player_id = player_id
            .ok_or_else(|| ClientRuntimeError::Config("--player-id is required".into()))?;
        if player_id == 0 {
            return Err(ClientRuntimeError::Config(
                "--player-id must be non-zero".into(),
            ));
        }
        if !matches!(presentation_hz, 30 | 60 | 120) {
            return Err(ClientRuntimeError::Config(
                "--presentation-hz must be 30, 60, or 120".into(),
            ));
        }
        if protocol_version != 2 {
            return Err(ClientRuntimeError::Config(
                "secure runtime requires protocol version 2".into(),
            ));
        }
        let presentation_bind = presentation_bind
            .ok_or_else(|| ClientRuntimeError::Config("--presentation-bind is required".into()))?;
        if !presentation_bind.ip().is_loopback() {
            return Err(ClientRuntimeError::Config(
                "presentation IPC must bind to a loopback address".into(),
            ));
        }
        Ok(Self {
            player_id,
            team_id,
            player_name: player_name.unwrap_or_else(|| format!("team-{team_id}-runtime")),
            server_addr: server_addr
                .ok_or_else(|| ClientRuntimeError::Config("--server is required".into()))?,
            presentation_bind,
            presentation_hz,
            evidence_dir,
            test_mode,
            protocol_version,
            content_hash,
            scripted_move_tick,
            scripted_hidden_target_tick,
            screenshot_tick,
            fault_tick,
            rebase_probe_tick,
            shutdown_file,
        })
    }
}

fn parse_u32(value: &str, flag: &str) -> Result<u32, ClientRuntimeError> {
    value
        .parse()
        .map_err(|_| ClientRuntimeError::Config(format!("invalid integer for {flag}")))
}

fn parse_addr(value: &str, flag: &str) -> Result<SocketAddr, ClientRuntimeError> {
    value
        .parse()
        .map_err(|_| ClientRuntimeError::Config(format!("invalid socket address for {flag}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_wrong_team_remote_ip_and_protocol_downgrade() {
        let base = |team: &str, bind: &str, protocol: &str| {
            vec![
                "--player-id",
                "1",
                "--team",
                team,
                "--server",
                "127.0.0.1:50061",
                "--presentation-bind",
                bind,
                "--protocol-version",
                protocol,
            ]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>()
        };
        assert!(ClientRuntimeConfig::parse(base("3", "127.0.0.1:62001", "2")).is_err());
        assert!(ClientRuntimeConfig::parse(base("1", "10.0.0.2:62001", "2")).is_err());
        assert!(ClientRuntimeConfig::parse(base("1", "127.0.0.1:62001", "1")).is_err());
    }

    #[test]
    fn parses_test_shutdown_file() {
        let args = vec![
            "--player-id",
            "1",
            "--team",
            "1",
            "--server",
            "127.0.0.1:50061",
            "--presentation-bind",
            "127.0.0.1:62001",
            "--protocol-version",
            "2",
            "--test-mode",
            "--evidence-dir",
            "evidence",
            "--shutdown-file",
            "shutdown.signal",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
        let config = ClientRuntimeConfig::parse(args).expect("shutdown-file config should parse");
        assert_eq!(config.shutdown_file, Some(PathBuf::from("shutdown.signal")));
        assert!(config.test_mode);
    }
}
