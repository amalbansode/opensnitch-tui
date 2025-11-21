//! Constants sourced from commit e5d1702bd36dd06c552c95ad3a3dcd023c3eb231
//! opensnitch/ui/opensnitch/config.py
//! opensnitch/daemon/rule/operator.go

/// Operand types to label firewall rules' data blob.
pub mod operand {
    pub const OPERAND_PROCESS_ID: &str = "process.id";
    pub const OPERAND_PROCESS_PATH: &str = "process.path";
    pub const OPERAND_PROCESS_COMMAND: &str = "process.command";
    pub const OPERAND_PROCESS_ENV: &str = "process.env.";
    pub const OPERAND_PROCESS_HASH_MD5: &str = "process.hash.md5";
    pub const OPERAND_PROCESS_HASH_SHA1: &str = "process.hash.sha1";
    pub const OPERAND_USER_ID: &str = "user.id";
    pub const OPERAND_IFACE_OUT: &str = "iface.out";
    pub const OPERAND_IFACE_IN: &str = "iface.in";
    pub const OPERAND_SOURCE_IP: &str = "source.ip";
    pub const OPERAND_SOURCE_PORT: &str = "source.port";
    pub const OPERAND_DEST_IP: &str = "dest.ip";
    pub const OPERAND_DEST_HOST: &str = "dest.host";
    pub const OPERAND_DEST_PORT: &str = "dest.port";
    pub const OPERAND_DEST_NETWORK: &str = "dest.network";
    pub const OPERAND_SOURCE_NETWORK: &str = "source.network";
    pub const OPERAND_PROTOCOL: &str = "protocol";
    pub const OPERAND_LIST: &str = "list";
    pub const OPERAND_LIST_DOMAINS: &str = "lists.domains";
    pub const OPERAND_LIST_DOMAINS_REGEXP: &str = "lists.domains_regexp";
    pub const OPERAND_LIST_IPS: &str = "lists.ips";
    pub const OPERAND_LIST_NETS: &str = "lists.nets";
}

/// Firewall rule type hint.
pub mod rule_type {
    pub const RULE_TYPE_LIST: &str = "list";
    pub const RULE_TYPE_LISTS: &str = "lists";
    pub const RULE_TYPE_SIMPLE: &str = "simple";
    pub const RULE_TYPE_REGEXP: &str = "regexp";
    pub const RULE_TYPE_NETWORK: &str = "network";
}

/// Firewall rule actions.
/// Note: A daemon's "default actions" set is a narrower subset
/// of this list, see below.
pub mod action {
    pub const ACTION_ALLOW: &str = "allow";
    pub const ACTION_DENY: &str = "deny";
    pub const ACTION_REJECT: &str = "reject";
    pub const ACTION_ACCEPT: &str = "accept";
    pub const ACTION_DROP: &str = "drop";
    pub const ACTION_JUMP: &str = "jump";
    pub const ACTION_REDIRECT: &str = "redirect";
    pub const ACTION_RETURN: &str = "return";
    pub const ACTION_TPROXY: &str = "tproxy";
    pub const ACTION_SNAT: &str = "snat";
    pub const ACTION_DNAT: &str = "dnat";
    pub const ACTION_MASQUERADE: &str = "masquerade";
    pub const ACTION_QUEUE: &str = "queue";
    pub const ACTION_LOG: &str = "log";
    pub const ACTION_STOP: &str = "stop";
}

/// Durations for firewall rules to be applicable.
#[allow(non_upper_case_globals)]
pub mod duration {
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
        pub fn new(s: &str) -> Result<Duration, ()> {
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
                _ => Err(()),
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
}

/// Default action values.
pub mod default_action {
    #[derive(Debug, Clone, Copy)]
    pub enum DefaultAction {
        Allow,
        Deny,
        Reject,
    }

    impl DefaultAction {
        /// Validates input action and returns enum variant.
        pub fn new(s: &str) -> Result<DefaultAction, ()> {
            match s {
                "allow" => Ok(DefaultAction::Allow),
                "deny" => Ok(DefaultAction::Deny),
                "reject" => Ok(DefaultAction::Reject),
                _ => Err(()),
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
}
