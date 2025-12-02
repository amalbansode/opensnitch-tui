use crate::opensnitch_proto::pb::Operator;
use serde::ser::{Serialize, SerializeStruct, Serializer};

impl Serialize for Operator {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Operator", 5)?;
        state.serialize_field("type", &self.r#type)?;
        state.serialize_field("operand", &self.operand)?;
        state.serialize_field("data", &self.data)?;
        state.serialize_field("sensitive", &self.sensitive)?;
        state.serialize_field("list", &self.list)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test Operator JSON serialization.
    #[test]
    fn test_operator_serialize() {
        let input = Operator {
            r#type: String::from("simple"),
            operand: String::from("protocol"),
            data: String::from("tcp"),
            sensitive: false,
            list: Vec::default(),
        };
        let output = serde_json::to_string(&input).expect("failed serialize struct to json");
        let expected_output = String::from(
            "{\"type\":\"simple\",\"operand\":\"protocol\",\
            \"data\":\"tcp\",\"sensitive\":false,\"list\":[]}",
        );
        assert_eq!(output, expected_output);
    }
}
