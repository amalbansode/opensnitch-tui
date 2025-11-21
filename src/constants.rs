//! Constants sourced from commit e5d1702bd36dd06c552c95ad3a3dcd023c3eb231
//! opensnitch/ui/opensnitch/config.py
//! opensnitch/daemon/rule/operator.go

/// Operand types to label firewall rules' data blob.
#[derive(Debug, Clone, Copy)]
pub enum Operand {
    ProcessId,
    ProcessPath,
    ProcessCmd,
    ProcessEnv,
    ProcessHashMd5,
    ProcessHashSha1,
    UserId,
    IfaceOut,
    IfaceIn,
    SrcIp,
    SrcPort,
    DstIp,
    DstHost,
    DstPort,
    DstNetwork,
    SrcNetwork,
    Protocol,
    List,
    ListDomains,
    ListDomainsRegexp,
    ListIps,
    ListNets,
}

impl Operand {
    /// Enum as string for OpenSnitch daemon.
    pub fn get_str(&self) -> &str {
        match self {
            Operand::ProcessId => "process.id",
            Operand::ProcessPath => "process.path",
            Operand::ProcessCmd => "process.command",
            Operand::ProcessEnv => "process.env.",
            Operand::ProcessHashMd5 => "process.hash.md5",
            Operand::ProcessHashSha1 => "process.hash.sha1",
            Operand::UserId => "user.id",
            Operand::IfaceOut => "iface.out",
            Operand::IfaceIn => "iface.in",
            Operand::SrcIp => "source.ip",
            Operand::SrcPort => "source.port",
            Operand::DstIp => "dest.ip",
            Operand::DstHost => "dest.host",
            Operand::DstPort => "dest.port",
            Operand::DstNetwork => "dest.network",
            Operand::SrcNetwork => "source.network",
            Operand::Protocol => "protocol",
            Operand::List => "list",
            Operand::ListDomains => "lists.domains",
            Operand::ListDomainsRegexp => "lists.domains_regexp",
            Operand::ListIps => "lists.ips",
            Operand::ListNets => "lists.nets",
        }
    }
}

/// Firewall rule type hint.
#[derive(Debug, Clone, Copy)]
pub enum RuleType {
    List,
    Lists,
    Simple,
    Regexp,
    Network,
}

impl RuleType {
    /// Enum as string for OpenSnitch daemon.
    pub fn get_str(&self) -> &str {
        match self {
            RuleType::List => "list",
            RuleType::Lists => "lists",
            RuleType::Simple => "simple",
            RuleType::Regexp => "regexp",
            RuleType::Network => "network",
        }
    }
}

/// Firewall rule actions.
/// Note: A daemon's "default actions" set is a narrower subset
/// of this list, see DefaultAction.
#[derive(Debug, Clone, Copy)]
pub enum Action {
    Allow,
    Deny,
    Reject,
    Accept,
    Drop,
    Jump,
    Redirect,
    Return,
    TProxy,
    Snat,
    Dnat,
    Masquerade,
    Queue,
    Log,
    Stop,
}

impl Action {
    /// Validates input action and returns enum variant.
    pub fn new(s: &str) -> Result<Action, BadOption> {
        match s {
            "allow" => Ok(Action::Allow),
            "deny" => Ok(Action::Deny),
            "reject" => Ok(Action::Reject),
            "accept" => Ok(Action::Accept),
            "drop" => Ok(Action::Drop),
            "jump" => Ok(Action::Jump),
            "redirect" => Ok(Action::Redirect),
            "return" => Ok(Action::Return),
            "tproxy" => Ok(Action::TProxy),
            "snat" => Ok(Action::Snat),
            "dnat" => Ok(Action::Dnat),
            "masquerade" => Ok(Action::Masquerade),
            "queue" => Ok(Action::Queue),
            "log" => Ok(Action::Log),
            "stop" => Ok(Action::Stop),
            _ => Err(BadOption {
                input: s.to_string(),
            }),
        }
    }

    /// Enum as string for OpenSnitch daemon.
    pub fn get_str(&self) -> &str {
        match self {
            Action::Allow => "allow",
            Action::Deny => "deny",
            Action::Reject => "reject",
            Action::Accept => "accept",
            Action::Drop => "drop",
            Action::Jump => "jump",
            Action::Redirect => "redirect",
            Action::Return => "return",
            Action::TProxy => "tproxy",
            Action::Snat => "snat",
            Action::Dnat => "dnat",
            Action::Masquerade => "masquerade",
            Action::Queue => "queue",
            Action::Log => "log",
            Action::Stop => "stop",
        }
    }
}

/// Durations for firewall rules to be applicable.
pub const DURATION_FIELD: &str = "duration";

#[derive(Debug, Clone, Copy)]
pub enum Duration {
    UntilRestart,
    Always,
    Once,
    Hours12,
    Hours1,
    Minutes30,
    Minutes15,
    Minutes5,
    Seconds30,
}

impl Duration {
    /// Validates input duration and returns enum variant.
    pub fn new(s: &str) -> Result<Duration, BadOption> {
        match s {
            "until restart" => Ok(Duration::UntilRestart),
            "always" => Ok(Duration::Always),
            "once" => Ok(Duration::Once),
            "12h" => Ok(Duration::Hours12),
            "1h" => Ok(Duration::Hours1),
            "30m" => Ok(Duration::Minutes30),
            "15m" => Ok(Duration::Minutes15),
            "5m" => Ok(Duration::Minutes5),
            "30s" => Ok(Duration::Seconds30),
            _ => Err(BadOption {
                input: s.to_string(),
            }),
        }
    }

    /// Enum as string for OpenSnitch daemon.
    pub fn get_str(&self) -> &str {
        match self {
            Duration::UntilRestart => "until restart",
            Duration::Always => "always",
            Duration::Once => "once",
            Duration::Hours12 => "12h",
            Duration::Hours1 => "1h",
            Duration::Minutes30 => "30m",
            Duration::Minutes15 => "15m",
            Duration::Minutes5 => "5m",
            Duration::Seconds30 => "30s",
        }
    }
}

/// Default action values.
#[derive(Debug, Clone, Copy)]
pub enum DefaultAction {
    Allow,
    Deny,
    Reject,
}

impl DefaultAction {
    /// Validates input action and returns enum variant.
    pub fn new(s: &str) -> Result<DefaultAction, BadOption> {
        match s {
            "allow" => Ok(DefaultAction::Allow),
            "deny" => Ok(DefaultAction::Deny),
            "reject" => Ok(DefaultAction::Reject),
            _ => Err(BadOption {
                input: s.to_string(),
            }),
        }
    }

    /// Enum as string for OpenSnitch daemon.
    pub fn get_str(&self) -> &str {
        match self {
            DefaultAction::Allow => "allow",
            DefaultAction::Deny => "deny",
            DefaultAction::Reject => "reject",
        }
    }
}

/// Error type for bad option provided to enum constructor.
#[derive(Debug, Clone)]
pub struct BadOption {
    pub input: String,
}

impl std::fmt::Display for BadOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bad Option: {}", self.input)
    }
}

impl std::error::Error for BadOption {}
