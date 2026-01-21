//! Command-line argument parsing

use std::path::PathBuf;

/// Command-line arguments
#[derive(Debug, Clone)]
pub struct CliArgs {
    /// Config file path
    pub config: Option<PathBuf>,
    /// MQTT host override
    pub mqtt_host: Option<String>,
    /// MQTT port override
    pub mqtt_port: Option<u16>,
    /// Player name override
    pub player_name: Option<String>,
    /// Enable verbose logging
    pub verbose: bool,
    /// Start paused
    pub start_paused: bool,
    /// Generate default config and exit
    pub generate_config: bool,
}

impl CliArgs {
    /// Parse command-line arguments
    pub fn parse() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut cli = Self {
            config: None,
            mqtt_host: None,
            mqtt_port: None,
            player_name: None,
            verbose: false,
            start_paused: false,
            generate_config: false,
        };

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--config" | "-c" => {
                    if i + 1 < args.len() {
                        cli.config = Some(PathBuf::from(&args[i + 1]));
                        i += 1;
                    }
                }
                "--mqtt-host" => {
                    if i + 1 < args.len() {
                        cli.mqtt_host = Some(args[i + 1].clone());
                        i += 1;
                    }
                }
                "--mqtt-port" => {
                    if i + 1 < args.len() {
                        cli.mqtt_port = args[i + 1].parse().ok();
                        i += 1;
                    }
                }
                "--player" | "-p" => {
                    if i + 1 < args.len() {
                        cli.player_name = Some(args[i + 1].clone());
                        i += 1;
                    }
                }
                "--verbose" | "-v" => cli.verbose = true,
                "--paused" => cli.start_paused = true,
                "--generate-config" => cli.generate_config = true,
                "--help" | "-h" => {
                    Self::print_help();
                    std::process::exit(0);
                }
                _ => {
                    log::warn!("Unknown argument: {}", args[i]);
                }
            }
            i += 1;
        }

        cli
    }

    fn print_help() {
        println!("OMFX - OMOBA Fyrox Debug Frontend");
        println!();
        println!("USAGE:");
        println!("    omfx [OPTIONS]");
        println!();
        println!("OPTIONS:");
        println!("    -c, --config <FILE>     Config file path");
        println!("    --mqtt-host <HOST>      MQTT broker host");
        println!("    --mqtt-port <PORT>      MQTT broker port");
        println!("    -p, --player <NAME>     Player name");
        println!("    -v, --verbose           Enable verbose logging");
        println!("    --paused                Start paused");
        println!("    --generate-config       Generate default config file");
        println!("    -h, --help              Print help");
    }
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            config: None,
            mqtt_host: None,
            mqtt_port: None,
            player_name: None,
            verbose: false,
            start_paused: false,
            generate_config: false,
        }
    }
}
