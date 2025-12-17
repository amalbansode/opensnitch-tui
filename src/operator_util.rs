use crate::opensnitch_proto::pb::Operator;

use crate::constants;
use std::str::FromStr;

/// A map representing which presets are enabled for operator chain creation.
/// TODO: wishlist - process hashes.
#[derive(Clone, Debug, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct PresetCombination {
    pub exact_user_id: bool,
    pub exact_ppath: bool,
    pub exact_dst_ip: bool,
    pub exact_dst_port: bool,
    pub exact_protocol: bool,
    pub match_hostname: MatchHostname,
}

/// Hostname matching isn't binary. It may be disabled or a degree of enabled: exact/any subdomain.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum MatchHostname {
    #[default]
    Disabled,
    Exact,
    AnySubdomain,
}

/// Util for CLI.
impl FromStr for PresetCombination {
    type Err = String;

    /// Parse a string of comma-delimited keys as a `PresetCombination`.
    /// TODO: Magic possible?
    /// # Errors
    /// If key doesn't match a known preset.
    fn from_str(s: &str) -> Result<PresetCombination, Self::Err> {
        let mut res = PresetCombination::default();
        for key in s.split(',') {
            match key {
                "exact_user_id" => {
                    res.exact_user_id = true;
                }
                "exact_ppath" => {
                    res.exact_ppath = true;
                }
                "exact_dst_ip" => {
                    res.exact_dst_ip = true;
                }
                "exact_dst_port" => {
                    res.exact_dst_port = true;
                }
                "exact_protocol" => {
                    res.exact_protocol = true;
                }
                "exact_hostname" => {
                    if res.match_hostname != MatchHostname::default() {
                        return Err("Conflicting hostname matching presets".to_string());
                    }
                    res.match_hostname = MatchHostname::Exact;
                }
                "any_subdomain_hostname" => {
                    if res.match_hostname != MatchHostname::default() {
                        return Err("Conflicting hostname matching presets".to_string());
                    }
                    res.match_hostname = MatchHostname::AnySubdomain;
                }
                _ => {
                    return Err(format!("Unknown preset key: {key}"));
                }
            }
        }
        Ok(res)
    }
}

/// A collection of all the inputs that may be used to generate chained preset operators.
/// All members are optional because some inputs may be unknown when considering a specific event/connection.
/// This may appear like a subset of `pb::Connection`, but we want to decouple the two so we may pave
/// the way towards a rule editor in the future.
#[derive(Debug, Default, Clone)]
pub struct GeneratedOperatorsInputs {
    pub user_id: Option<u32>,
    pub ppath: Option<String>,
    pub dst_ip: Option<String>,
    pub dst_port: Option<u32>,
    pub protocol: Option<String>,
    pub hostname: Option<String>,
}

impl GeneratedOperatorsInputs {
    /// Generate preset operators for this input struct based on the desired preset combo.
    #[must_use]
    pub fn generate_operators(&self, combo: &PresetCombination) -> Vec<Operator> {
        let mut res = Vec::<Operator>::default();
        // If a preset is enabled && data for that preset exists in inputs, add.
        // If a preset is enabled but data for that preset doesn't exist in inputs,
        // silently drop the operator for that data field. This could be up for debate.
        if combo.exact_user_id
            && let Some(user_id) = self.user_id
        {
            res.push(match_user_id(user_id));
        }
        if combo.exact_ppath
            && let Some(ppath) = &self.ppath
        {
            res.push(match_proc_path(ppath));
        }
        if combo.exact_dst_ip
            && let Some(dst_ip) = &self.dst_ip
        {
            res.push(match_dst_ip(dst_ip));
        }
        if combo.exact_dst_port
            && let Some(dst_port) = self.dst_port
        {
            res.push(match_dst_port(dst_port));
        }
        if combo.exact_protocol
            && let Some(protocol) = &self.protocol
        {
            res.push(match_protocol(protocol));
        }
        match combo.match_hostname {
            MatchHostname::Disabled => {}
            MatchHostname::Exact => {
                if let Some(hostname) = &self.hostname {
                    res.push(match_exact_hostname(hostname));
                }
            }
            MatchHostname::AnySubdomain => {
                if let Some(hostname) = &self.hostname {
                    res.push(match_any_subdomain_hostname(hostname));
                }
            }
        }
        res
    }
}

#[must_use]
fn match_user_id(uid: u32) -> Operator {
    Operator {
        r#type: String::from(constants::RuleType::Simple.get_str()),
        operand: String::from(constants::Operand::UserId.get_str()),
        data: uid.to_string(),
        sensitive: false,
        list: Vec::default(),
    }
}

#[must_use]
fn match_proc_path(ppath: &str) -> Operator {
    Operator {
        r#type: String::from(constants::RuleType::Simple.get_str()),
        operand: String::from(constants::Operand::ProcessPath.get_str()),
        data: ppath.to_owned(),
        sensitive: false,
        list: Vec::default(),
    }
}

#[must_use]
fn match_dst_ip(ip: &str) -> Operator {
    Operator {
        r#type: String::from(constants::RuleType::Simple.get_str()),
        operand: String::from(constants::Operand::DstIp.get_str()),
        data: ip.to_owned(),
        sensitive: false,
        list: Vec::default(),
    }
}

#[must_use]
fn match_dst_port(port: u32) -> Operator {
    Operator {
        r#type: String::from(constants::RuleType::Simple.get_str()),
        operand: String::from(constants::Operand::DstPort.get_str()),
        data: port.to_string(),
        sensitive: false,
        list: Vec::default(),
    }
}

#[must_use]
fn match_protocol(protocol: &str) -> Operator {
    Operator {
        r#type: String::from(constants::RuleType::Simple.get_str()),
        operand: String::from(constants::Operand::Protocol.get_str()),
        data: protocol.to_owned(),
        sensitive: false,
        list: Vec::default(),
    }
}

/// Match exact hostname.
#[must_use]
fn match_exact_hostname(hostname: &str) -> Operator {
    Operator {
        r#type: String::from(constants::RuleType::Simple.get_str()),
        operand: String::from(constants::Operand::DstHost.get_str()),
        data: hostname.to_owned(),
        sensitive: false,
        list: Vec::default(),
    }
}

/// Match any subdomain of hostname and hostname itself.
#[must_use]
fn match_any_subdomain_hostname(hostname: &str) -> Operator {
    // Escape dots, add start+end and exact/subdomain matchers.
    let escaped = hostname.replace('.', "\\.");
    let re = format!("(?:^|\\.){escaped}$");
    Operator {
        r#type: String::from(constants::RuleType::Regexp.get_str()),
        operand: String::from(constants::Operand::DstHost.get_str()),
        data: re,
        sensitive: false,
        list: Vec::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use std::collections::HashMap;

    /// Sanity test regex for hostname subdomains matching.
    #[test]
    fn test_match_any_subdomain_hostname() {
        // Map of <Test key> to <whether match expected>
        let mut tests = HashMap::new();
        tests.insert("example.com", true);
        tests.insert("www.example.com", true);
        tests.insert("w1.w2.example.com", true);
        tests.insert("bexample.com", false);
        tests.insert("example-com.com", false);
        tests.insert("hello.bexample.com", false);

        // Test subdomain matching on "example.com"
        let op = match_any_subdomain_hostname(&"example.com".to_string());
        assert_eq!(op.r#type, "regexp");
        assert_eq!(op.operand, "dest.host");
        assert_eq!(op.sensitive, false);
        assert_eq!(op.list, Vec::default());

        let test_re = Regex::new(op.data.as_str()).expect("Bad regex");
        for (test_key, match_expected) in tests {
            println!("Testing: {test_key} expected: {match_expected}");
            assert_eq!(test_re.is_match(test_key), match_expected);
        }
    }

    /// Test operator generator for a single op.
    #[test]
    fn test_generator_single() {
        let input = GeneratedOperatorsInputs {
            user_id: Some(123),
            ppath: Some("/bin/nc".to_string()),
            dst_ip: Some("1.2.3.4".to_string()),
            dst_port: Some(567),
            protocol: Some("tcp".to_string()),
            hostname: None,
        };

        let mut combo = PresetCombination::default();
        combo.exact_dst_ip = true;

        let expected_out = vec![Operator {
            r#type: String::from(constants::RuleType::Simple.get_str()),
            operand: String::from(constants::Operand::DstIp.get_str()),
            data: "1.2.3.4".to_string(),
            sensitive: false,
            list: Vec::default(),
        }];

        let out = input.generate_operators(&combo);

        assert_eq!(out, expected_out);
    }

    /// Test operator generator for many ops.
    #[test]
    fn test_generator_many() {
        let input = GeneratedOperatorsInputs {
            user_id: Some(123),
            ppath: Some("/bin/nc".to_string()),
            dst_ip: Some("1.2.3.4".to_string()),
            dst_port: Some(567),
            protocol: Some("tcp".to_string()),
            hostname: None,
        };

        let mut combo = PresetCombination::default();
        combo.exact_dst_ip = true;
        combo.exact_dst_port = true;

        let expected_out = vec![
            Operator {
                r#type: String::from(constants::RuleType::Simple.get_str()),
                operand: String::from(constants::Operand::DstIp.get_str()),
                data: "1.2.3.4".to_string(),
                sensitive: false,
                list: Vec::default(),
            },
            Operator {
                r#type: String::from(constants::RuleType::Simple.get_str()),
                operand: String::from(constants::Operand::DstPort.get_str()),
                data: "567".to_string(),
                sensitive: false,
                list: Vec::default(),
            },
        ];

        let out = input.generate_operators(&combo);

        assert_eq!(out, expected_out);
    }

    /// Test operator generator for a single op, but missing data.
    /// TODO: Ideally test this with every input, but I'm lazy.
    #[test]
    fn test_generator_single_missing_data() {
        let input = GeneratedOperatorsInputs {
            user_id: Some(123),
            ppath: Some("/bin/nc".to_string()),
            dst_ip: Some("1.2.3.4".to_string()),
            dst_port: Some(567),
            protocol: Some("tcp".to_string()),
            hostname: None,
        };

        let mut combo = PresetCombination::default();
        combo.match_hostname = MatchHostname::Exact;

        let expected_out = vec![];

        let out = input.generate_operators(&combo);

        assert_eq!(out, expected_out);
    }
}
