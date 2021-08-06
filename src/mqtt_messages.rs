use lazy_static::lazy_static;
use matchit::Node;
use paho_mqtt::Message;
use serde::Deserialize;
use std::num::NonZeroU8;
use tracing::{event, Level};

lazy_static! {
  static ref MATCHER: Node<MqttMessageKind> = {
    let mut node = Node::new();
    node
      .insert(
        "homeassistant/light/enet-1/:number/set",
        MqttMessageKind::SetDeviceState,
      )
      .unwrap();
    node
  };
}

pub enum MqttMessage {
  SetDeviceState(u32, HassDeviceState),
}

impl MqttMessage {
  pub fn try_match(message: Message) -> Option<Self> {
    let result = match MATCHER.at(message.topic()) {
      Ok(v) => v,
      Err(_) => {
        event!(
          Level::INFO,
          message.topic = message.topic(),
          "Received MQTT message on unmatched topic.",
        );

        return None;
      }
    };

    match result.value {
      MqttMessageKind::SetDeviceState => {
        let num: u32 = match result.params.get("number").unwrap().parse() {
          Ok(num) => num,
          Err(e) => {
            event!(
              Level::WARN,
              message.topic = message.topic(),
              "Received MQTT message where topic did not parse correctly: {:?}",
              e,
            );

            return None;
          }
        };

        let body: HassDeviceState = match serde_json::from_slice(message.payload()) {
          Ok(body) => body,
          Err(e) => {
            event!(
              Level::WARN,
              message.topic = message.topic(),
              "Failed to parse MQTT message body: {:?}",
              e,
            );

            return None;
          }
        };

        Some(Self::SetDeviceState(num, body))
      }
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MqttMessageKind {
  SetDeviceState,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "UPPERCASE")]
pub enum HassDeviceState {
  On(HassOnState),
  Off,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct HassOnState {
  #[serde(default)]
  brightness: Option<NonZeroU8>,
}

impl From<HassDeviceState> for enet_client::SetValue {
  fn from(state: HassDeviceState) -> Self {
    match state {
      HassDeviceState::On(HassOnState { brightness: None }) => {
        Self::On(enet_client::ClickDuration::Short)
      }
      HassDeviceState::On(HassOnState {
        brightness: Some(v),
      }) => Self::Dimm(from_full_byte(v.get())),
      HassDeviceState::Off => Self::Off(enet_client::ClickDuration::Short),
    }
  }
}

fn from_full_byte(value: u8) -> u8 {
  let v = value as u32;
  let p = v * 100u32 / 255u32;
  p as u8
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_test::{assert_de_tokens, Token};

  #[test]
  fn from_full_byte_test() {
    assert_eq!(0, from_full_byte(0), "0");
    assert_eq!(100, from_full_byte(255), "255");
  }

  #[test]
  fn deserialize_off_state() {
    assert_de_tokens(
      &HassDeviceState::Off,
      &[
        Token::Map { len: Some(1) },
        Token::Str("state"),
        Token::Str("OFF"),
        Token::MapEnd,
      ],
    )
  }

  #[test]
  fn deserialize_on_without_brightness() {
    assert_de_tokens(
      &HassDeviceState::On(HassOnState { brightness: None }),
      &[
        Token::Map { len: Some(1) },
        Token::Str("state"),
        Token::Str("ON"),
        Token::MapEnd,
      ],
    )
  }

  #[test]
  fn deserialize_on_with_brightness() {
    assert_de_tokens(
      &HassDeviceState::On(HassOnState {
        brightness: NonZeroU8::new(100u8),
      }),
      &[
        Token::Map { len: Some(2) },
        Token::Str("state"),
        Token::Str("ON"),
        Token::Str("brightness"),
        Token::U8(100),
        Token::MapEnd,
      ],
    )
  }
}
