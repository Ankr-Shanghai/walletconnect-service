use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct WalletMessage {
    #[serde(rename = "type")]
    pub kind: MessageKind,
    pub topic: String,
    pub payload: String,
    pub silent: bool,
}

impl std::fmt::Display for WalletMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\"type\": \"{}\", \"topic\": \"{}\", \"payload\": \"{}\"}}",
            self.kind.as_str(),
            self.topic,
            self.payload
        )
    }
}

impl redis::ToRedisArgs for WalletMessage {
    fn write_redis_args<W: ?Sized>(&self, out: &mut W)
    where
        W: redis::RedisWrite,
    {
        let s = self.to_string();
        out.write_arg(s.as_bytes());
    }
}

pub enum MessageKind {
    Pub,
    Sub,
}

use super::super::constant;
impl MessageKind {
    fn as_str(&self) -> &str {
        match self {
            MessageKind::Pub => constant::PUB_MSG_TYPE,
            MessageKind::Sub => constant::SUB_MSG_TYPE,
        }
    }
}

impl std::fmt::Display for MessageKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Serialize for MessageKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.as_str();
        serializer.serialize_str(s)
    }
}

impl<'a> Deserialize<'a> for MessageKind {
    fn deserialize<D>(deserializer: D) -> Result<MessageKind, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            constant::PUB_MSG_TYPE => Ok(MessageKind::Pub),
            constant::SUB_MSG_TYPE => Ok(MessageKind::Sub),
            _ => Err(serde::de::Error::custom(format!(
                "invalid message type: {}",
                s
            ))),
        }
    }
}
