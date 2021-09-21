use enet_client::{ClickDuration, SetValue};
use lazy_static::lazy_static;
use matchit::Node;
use paho_mqtt::Message;
use std::str::FromStr;
use tracing::{event, Level};

lazy_static! {
  static ref MATCHER: Node<MqttMessageKind> = {
    let mut node = Node::new();
    node
      .insert(
        "enet/instance-1/:number/set",
        MqttMessageKind::SetDeviceState,
      )
      .unwrap();
    node
      .insert(
        "enet/instance-1/:number/bri/set",
        MqttMessageKind::SetDeviceBrightness,
      )
      .unwrap();
    node
  };
}

pub enum MqttMessage {
  SetDeviceState(u32, HassDeviceState),
  SetDeviceBrightness(u32, u8),
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
      MqttMessageKind::SetDeviceBrightness => {
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

        let body = match message.payload_str().parse() {
          Ok(v) => v,
          Err(e) => {
            event!(
              Level::WARN,
              message.topic = message.topic(),
              "Failed to parse MQTT message body: {:?}\nbody:\n{}",
              e,
              message.payload_str(),
            );

            return None;
          }
        };

        Some(Self::SetDeviceBrightness(num, body))
      }

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

        let body = match message.payload_str().parse() {
          Ok(v) => v,
          Err(e) => {
            event!(
              Level::WARN,
              message.topic = message.topic(),
              "Failed to parse MQTT message body: {:?}\nbody:\n{}",
              e,
              message.payload_str(),
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
  SetDeviceBrightness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HassDeviceState {
  On,
  Off,
}

impl FromStr for HassDeviceState {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "ON" => Ok(HassDeviceState::On),
      "OFF" => Ok(HassDeviceState::Off),
      _ => Err(format!(
        "'{}' is not a valid state string (expected ON or OFF)",
        s
      )),
    }
  }
}

pub(crate) trait IntoDeviceValue {
  fn into_device_value(self) -> SetValue;
}

impl IntoDeviceValue for HassDeviceState {
  fn into_device_value(self) -> SetValue {
    match self {
      HassDeviceState::On => SetValue::On(ClickDuration::Short),
      HassDeviceState::Off => SetValue::Off(ClickDuration::Short),
    }
  }
}

impl IntoDeviceValue for u8 {
  fn into_device_value(self) -> SetValue {
    match self {
      0 => SetValue::Off(ClickDuration::Short),
      v => SetValue::Dimm(v),
    }
  }
}
