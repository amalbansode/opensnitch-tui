use crate::opensnitch_proto::pb::Operator;

use crate::constants;

pub fn match_user_id(uid: u32) -> Operator {
    Operator {
        r#type: String::from(constants::RuleType::Simple.get_str()),
        operand: String::from(constants::Operand::UserId.get_str()),
        data: uid.to_string(),
        sensitive: false,
        list: Vec::default(),
    }
}

pub fn match_proc_path(ppath: &str) -> Operator {
    Operator {
        r#type: String::from(constants::RuleType::Simple.get_str()),
        operand: String::from(constants::Operand::ProcessPath.get_str()),
        data: ppath.to_owned(),
        sensitive: false,
        list: Vec::default(),
    }
}

pub fn match_dst_ip(ip: &str) -> Operator {
    Operator {
        r#type: String::from(constants::RuleType::Simple.get_str()),
        operand: String::from(constants::Operand::DstIp.get_str()),
        data: ip.to_owned(),
        sensitive: false,
        list: Vec::default(),
    }
}

pub fn match_dst_port(port: u32) -> Operator {
    Operator {
        r#type: String::from(constants::RuleType::Simple.get_str()),
        operand: String::from(constants::Operand::DstPort.get_str()),
        data: port.to_string(),
        sensitive: false,
        list: Vec::default(),
    }
}

pub fn match_protocol(protocol: &str) -> Operator {
    Operator {
        r#type: String::from(constants::RuleType::Simple.get_str()),
        operand: String::from(constants::Operand::Protocol.get_str()),
        data: protocol.to_owned(),
        sensitive: false,
        list: Vec::default(),
    }
}
