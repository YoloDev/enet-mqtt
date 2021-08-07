mod args;
mod evlist;
mod mqtt_messages;

use args::{Args, LogFormat};
use async_stream::stream;
use clap::Clap;
use color_eyre::{eyre::Context, Report};
use enet_client::{
  dev::{Device, DeviceKind, DeviceValue, OnValue},
  EnetClient,
};
use evlist::EvList;
use futures::{pin_mut, stream::select, Stream, StreamExt};
use paho_mqtt::{AsyncClient, CreateOptions, Message};
use serde_json::json;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use crate::mqtt_messages::MqttMessage;

#[tokio::main]
async fn main() -> Result<(), Report> {
  let args = Args::parse();
  setup(args.log_format)?;

  info!("Connecting to eNet gateway {}.", args.gateway);
  let mut client = EnetClient::new(args.gateway).await?;
  info!("eNet client ready.");

  info!(
    "Connecting to MQTT broker {}:{} with user '{}' and has-password: {}",
    args.mqtt.host,
    args.mqtt.port,
    args.mqtt.auth.username.as_deref().unwrap_or("<no user>"),
    args.mqtt.auth.password.is_some(),
  );
  let mut mqtt =
    AsyncClient::new(CreateOptions::new()).wrap_err("Failed to create mqtt client.")?;
  let connect_options = args
    .mqtt
    .into_connect_options()
    .await
    .wrap_err("Failed to create connect options.")?;
  mqtt
    .connect(connect_options)
    .await
    .wrap_err("Failed to connect to mqtt.")?;

  let (sender, receiver) = unbounded_channel();
  mqtt.set_message_callback(move |_, msg| {
    if let Some(msg) = msg {
      let _ = sender.send(msg);
    }
  });
  mqtt
    .subscribe("homeassistant/light/enet-1/+/set", 2)
    .await
    .wrap_err("Failed to subscribe to topic.")?;

  let mut subscriptions = Vec::new();
  for device in client.devices() {
    info!(device.name = %device.name(), device.number = device.number(), "found device");
    subscriptions.push((device.clone(), device.subscribe()));

    let config_msg = match device.kind() {
      DeviceKind::Binary => {
        let topic = format!("homeassistant/light/enet-1/{}/config", device.number());
        let config = json!({
          "name": device.name(),
          "cmd_t": format!("homeassistant/light/enet-1/{}/set", device.number()),
          "stat_t": format!("homeassistant/light/enet-1/{}/state", device.number()),
          "schema": "json",
          "brightness": false,
          "channel": device.number(),
          "kind": "binary",
        });

        let bytes = serde_json::to_vec(&config).unwrap();
        Message::new_retained(topic, bytes, 2)
      }
      DeviceKind::Dimmer => {
        let topic = format!("homeassistant/light/enet-1/{}/config", device.number());
        let config = json!({
          "name": device.name(),
          "cmd_t": format!("homeassistant/light/enet-1/{}/set", device.number()),
          "stat_t": format!("homeassistant/light/enet-1/{}/state", device.number()),
          "schema": "json",
          "brightness": true,
          "channel": device.number(),
          "kind": "dimmer",
        });

        let bytes = serde_json::to_vec(&config).unwrap();
        Message::new_retained(topic, bytes, 2)
      }
      DeviceKind::Blinds => continue,
    };

    mqtt.publish(config_msg).await?;
  }

  let device_events = EvList::new(subscriptions).map(ApplicationMessage::from);
  let mqtt_events = channel_to_stream(receiver).map(ApplicationMessage::from);
  let events = select(device_events, mqtt_events);
  pin_mut!(events);

  while let Some(evt) = events.next().await {
    match evt {
      ApplicationMessage::DeviceUpdate(device, value) => {
        info!("{} updated to {}", device.name(), value);

        let state_msg = match device.kind() {
          DeviceKind::Binary => {
            let topic = format!("homeassistant/light/enet-1/{}/state", device.number());
            let config = json!({
              "state": if value.is_on() { "ON" } else { "OFF" }
            });

            let bytes = serde_json::to_vec(&config).unwrap();
            Message::new_retained(topic, bytes, 2)
          }
          DeviceKind::Dimmer => {
            let topic = format!("homeassistant/light/enet-1/{}/state", device.number());
            let config = json!({
              "state": if value.is_on() { "ON" } else { "OFF" },
              "brightness": to_full_byte(value.value())
            });

            let bytes = serde_json::to_vec(&config).unwrap();
            Message::new_retained(topic, bytes, 2)
          }
          DeviceKind::Blinds => continue,
        };

        mqtt.publish(state_msg).await?;
      }

      ApplicationMessage::MqttMessage(msg) => {
        info!(msg.topic = msg.topic(), "MQTT message received.");

        if let Some(msg) = MqttMessage::try_match(msg) {
          match msg {
            MqttMessage::SetDeviceState(number, value) => {
              info!("Set device state command received.");
              match client.device(number) {
                Some(_) => (),
                None => {
                  warn!("Got set-value command for unknown device number {}", number);
                  continue;
                }
              };

              match client.set_value(number, value.into()).await {
                Ok(_) => (),
                Err(e) => {
                  warn!("Failed to set value: {:?}", e);
                }
              }
            }
          }
        }
      }
    }
  }

  Ok(())
}

fn setup(format: LogFormat) -> Result<(), Report> {
  if std::env::var("RUST_LIB_BACKTRACE").is_err() {
    std::env::set_var("RUST_LIB_BACKTRACE", "1")
  }
  color_eyre::install()?;

  let filter = EnvFilter::from_default_env()
    // Set the base level when not matched by other directives to INFO.
    .add_directive(tracing::Level::INFO.into());
  match format {
    LogFormat::Pretty => {
      tracing_subscriber::fmt().with_env_filter(filter).init();
    }

    LogFormat::Json => {
      tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .with_current_span(false)
        .with_span_list(false)
        .init();
    }
  }

  Ok(())
}

fn to_full_byte(value: Option<OnValue>) -> u8 {
  match value {
    None => 0,
    Some(v) => {
      let v = v.get() as u32;
      let full_v = (v * (u8::MAX as u32)) / 100;
      full_v as u8
    }
  }
}

enum ApplicationMessage {
  DeviceUpdate(Device, DeviceValue),
  MqttMessage(Message),
}

impl From<(Device, DeviceValue)> for ApplicationMessage {
  #[inline]
  fn from((device, value): (Device, DeviceValue)) -> Self {
    Self::DeviceUpdate(device, value)
  }
}

impl From<Message> for ApplicationMessage {
  #[inline]
  fn from(message: Message) -> Self {
    Self::MqttMessage(message)
  }
}

fn channel_to_stream<T>(mut rx: UnboundedReceiver<T>) -> impl Stream<Item = T> {
  stream! {
    while let Some(item) = rx.recv().await {
      yield item;
    }
  }
}
