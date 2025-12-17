use std::time;

use crate::opensnitch_proto;

#[derive(Clone, Debug)]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    #[must_use]
    pub fn new(v: i32) -> Priority {
        match v {
            0 => Priority::Low,
            1 => Priority::Medium,
            _ => Priority::High,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Type {
    Error,
    Warning,
    Info,
}

impl Type {
    #[must_use]
    pub fn new(v: i32) -> Type {
        match v {
            0 => Type::Error,
            1 => Type::Warning,
            _ => Type::Info,
        }
    }
}

#[derive(Clone, Debug)]
pub enum What {
    Generic,
    ProcMonitor,
    Firewall,
    Connection,
    Rule,
    Netlink,
    KernelEvent,
}

impl What {
    #[must_use]
    pub fn new(v: i32) -> What {
        match v {
            // 0 is also generic, but caught by wildcard below.
            1 => What::ProcMonitor,
            2 => What::Firewall,
            3 => What::Connection,
            4 => What::Rule,
            5 => What::Netlink,
            6 => What::KernelEvent,
            _ => What::Generic,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Alert {
    pub timestamp: time::SystemTime,
    pub priority: Priority,
    pub r#type: Type,
    pub what: What,
    pub msg: String,
}

impl Alert {
    /// Create an alert keyed off protobuf message.
    #[must_use]
    pub fn from_proto(ts: time::SystemTime, proto: &opensnitch_proto::pb::Alert) -> Alert {
        let msg = match &proto.data {
            Some(data) => match data {
                opensnitch_proto::pb::alert::Data::Text(v) => v.clone(),
                _ => String::from("unsupported alert data"),
            },
            None => String::from("no data"),
        };

        Alert {
            timestamp: ts,
            priority: Priority::new(proto.priority),
            r#type: Type::new(proto.r#type),
            what: What::new(proto.what),
            msg,
        }
    }

    /// Create a simple alert.
    #[must_use]
    pub fn create_simple(ts: time::SystemTime, msg: &str) -> Alert {
        Alert {
            timestamp: ts,
            priority: Priority::Low,
            r#type: Type::Warning,
            what: What::Generic,
            msg: msg.to_string(),
        }
    }
}
