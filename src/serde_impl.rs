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
