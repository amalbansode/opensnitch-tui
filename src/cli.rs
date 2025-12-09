use clap::{Arg, Command};

#[must_use]
pub fn setup() -> Command {
    Command::new("opensnitch-tui")
    .author("Amal Bansode")
    .version("0.0.2")
    .about("A Terminal UI control plane for OpenSnitch.")
    .arg(
        Arg::new("ip_port")
        .long("bind")
        .default_value("127.0.0.1:50051")
        .help("IP address and port for OpenSnitch gRPC server to bind to. Format: \"A.B.C.D:port\" or \"[A:B:C::D]:port\".")
    )
    .arg(
        Arg::new("dispo_seconds")
        .long("conn-dispo-timeout")
        .default_value("30")
        .value_parser(clap::value_parser!(u64).range(1..115))
        .help("Duration in seconds that the TUI will wait on a disposition (allow/deny) for a connection attempt. Upon timeout, daemon will perform default action. Max: 115.")
    )
    .arg(
        Arg::new("default_action")
        .long("default-action")
        .default_value("deny")
        .help("Default action (allow/deny/reject) to be conveyed to daemons when the TUI fails to disposition a connection attempt in time.")
    )
    .arg(
        Arg::new("temp_rule_lifetime")
        .long("temp-rule-lifetime")
        .value_parser(["until restart", "always", "once", "12h", "1h", "30m", "15m", "5m", "30s",]) // TODO: Single source of truth from constants.rs?
        .default_value("12h")
        .help("Lifetime of temporary rules created by TUI.")
    )
    .max_term_width(100)
}
