/// Constants sourced from commit e5d1702bd36dd06c552c95ad3a3dcd023c3eb231
/// opensnitch/ui/opensnitch/config.py
/// opensnitch/daemon/rule/operator.go

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

pub mod rule_type {
    pub const RULE_TYPE_LIST: &str = "list";
    pub const RULE_TYPE_LISTS: &str = "lists";
    pub const RULE_TYPE_SIMPLE: &str = "simple";
    pub const RULE_TYPE_REGEXP: &str = "regexp";
    pub const RULE_TYPE_NETWORK: &str = "network";
}

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

#[allow(non_upper_case_globals)]
pub mod duration {
    pub const DURATION_FIELD: &str = "duration";
    pub const DURATION_UNTIL_RESTART: &str = "until restart";
    pub const DURATION_ALWAYS: &str = "always";
    pub const DURATION_ONCE: &str = "once";
    pub const DURATION_12h: &str = "12h";
    pub const DURATION_1h: &str = "1h";
    pub const DURATION_30m: &str = "30m";
    pub const DURATION_15m: &str = "15m";
    pub const DURATION_5m: &str = "5m";
    pub const DURATION_30s: &str = "30s";
}
