use crate::opensnitch_proto::pb::Operator;

use crate::constants;

pub fn match_user_id(uid: u32) -> Operator {
    Operator {
        r#type: String::from(constants::rule_type::RULE_TYPE_SIMPLE),
        operand: String::from(constants::operand::OPERAND_USER_ID),
        data: uid.to_string(),
        sensitive: false,
        list: Vec::default(),
    }
}

pub fn match_proc_path(ppath: &String) -> Operator {
    Operator {
        r#type: String::from(constants::rule_type::RULE_TYPE_SIMPLE),
        operand: String::from(constants::operand::OPERAND_PROCESS_PATH),
        data: ppath.clone(),
        sensitive: false,
        list: Vec::default(),
    }
}

pub fn match_dst_ip(ip: &String) -> Operator {
    Operator {
        r#type: String::from(constants::rule_type::RULE_TYPE_SIMPLE),
        operand: String::from(constants::operand::OPERAND_DEST_IP),
        data: ip.clone(),
        sensitive: false,
        list: Vec::default(),
    }
}

pub fn match_dst_port(port: u32) -> Operator {
    Operator {
        r#type: String::from(constants::rule_type::RULE_TYPE_SIMPLE),
        operand: String::from(constants::operand::OPERAND_DEST_PORT),
        data: port.to_string(),
        sensitive: false,
        list: Vec::default(),
    }
}

pub fn match_protocol(protocol: &String) -> Operator {
    Operator {
        r#type: String::from(constants::rule_type::RULE_TYPE_SIMPLE),
        operand: String::from(constants::operand::OPERAND_PROTOCOL),
        data: protocol.clone(),
        sensitive: false,
        list: Vec::default(),
    }
}
