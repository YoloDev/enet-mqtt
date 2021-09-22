mod args;
mod evlist;
mod mqtt_messages;

use std::{
  collections::HashMap,
  future::{ready, Ready},
  time::{Duration, SystemTime},
};

use args::{Args, LogFormat};
use async_stream::stream;
use clap::Clap;
use color_eyre::{eyre::Context, Report};
use enet_client::{
  dev::{Device, DeviceBrightness, DeviceState},
  EnetClient, EnetDevice, SetValue,
};
use eventuals::EventualExt;
use evlist::EvList;
use futures::{pin_mut, stream::select, Stream, StreamExt};
use paho_mqtt::{AsyncClient, CreateOptions, Message};
use serde_json::json;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use crate::mqtt_messages::{IntoDeviceValue, MqttMessage};

#[tokio::main]
async fn main() -> Result<(), Report> {
  _main().await
}

async fn _main() -> Result<(), Report> {
  let args = Args::parse();
  setup(args.log_format)?;

  info!("Connecting to eNet gateway {}.", args.gateway);
  let mut client = EnetClient::new(args.gateway.clone()).await?;
  info!("eNet client ready.");

  let online_msg = Message::new_retained("enet/instance-1/status", "online", 2);
  let offline_msg = Message::new_retained("enet/instance-1/status", "offline", 2);
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
    .into_connect_options(Some(offline_msg))
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
    .subscribe("enet/instance-1/+/set", 2)
    .await
    .wrap_err("Failed to subscribe to topic.")?;
  mqtt
    .subscribe("enet/instance-1/+/bri/set", 2)
    .await
    .wrap_err("Failed to subscribe to topic.")?;

  let mut subscriptions = Vec::new();
  for device in client.devices() {
    info!(device.name = %device.name(), device.number = device.number(), "found device");

    let (config_msg, attr_msg) = match device {
      Device::Binary(binary) => {
        subscriptions.push((
          device.clone(),
          binary
            .subscribe_state()
            .map(DeviceUpdate::state)
            .subscribe(),
        ));

        let topic = format!("homeassistant/light/enet-1/{}/config", binary.number());
        let config = json!({
          "~": format!("enet/instance-1/{}", binary.number()),
          "uniq_id": format!("enet-1-{}", binary.number()),
          "name": binary.name(),
          "cmd_t": "~/set",
          "stat_t": "~/state",
          "json_attr_t": "~/attr",
          "pl_on": "ON",
          "pl_off": "OFF",
          "pl_avail": "online",
          "pl_not_avail": "offline",
          "qos": 1,
          "device": {
            "ids": format!("enet-1-{}", binary.number()),
            "name": binary.name(),
          },
          "avty_t": "enet/instance-1/status",
        });

        let attr = json!({
          "channel": binary.number(),
          "kind": "binary",
          "enet_gateway": args.gateway,
          "assumed_state": false,
        });

        (
          msg_retained(topic, &config),
          msg_retained(format!("enet/instance-1/{}/attr", binary.number()), &attr),
        )
      }

      Device::Dimmer(dimmer) => {
        subscriptions.push((
          device.clone(),
          dimmer
            .subscribe_state()
            .map(DeviceUpdate::state)
            .subscribe(),
        ));
        subscriptions.push((
          device.clone(),
          dimmer
            .subscribe_brightness()
            .map(DeviceUpdate::brightness)
            .subscribe(),
        ));

        let topic = format!("homeassistant/light/enet-1/{}/config", dimmer.number());
        let config = json!({
          "~": format!("enet/instance-1/{}", dimmer.number()),
          "uniq_id": format!("enet-1-{}", dimmer.number()),
          "name": dimmer.name(),
          "cmd_t": "~/set",
          "stat_t": "~/state",
          "json_attr_t": "~/attr",
          "bri_stat_t": "~/bri",
          "bri_cmd_t": "~/bri/set",
          "bri_scl": "100",
          "pl_on": "ON",
          "pl_off": "OFF",
          "pl_avail": "online",
          "pl_not_avail": "offline",
          "qos": 1,
          "device": {
            "ids": format!("enet-1-{}", dimmer.number()),
            "name": dimmer.name(),
          },
          "avty_t": "enet/instance-1/status",
        });

        let attr = json!({
          "channel": dimmer.number(),
          "kind": "dimmer",
          "enet_gateway": args.gateway,
          "assumed_state": false,
        });

        (
          msg_retained(topic, &config),
          msg_retained(format!("enet/instance-1/{}/attr", dimmer.number()), &attr),
        )
      }
    };

    mqtt.publish(attr_msg).await?;
    mqtt.publish(config_msg).await?;
  }

  let device_events = EvList::new(subscriptions).map(ApplicationMessage::from);
  let mqtt_events = channel_to_stream(receiver).map(ApplicationMessage::from);
  let events = select(device_events, mqtt_events);
  pin_mut!(events);

  let mut bris: HashMap<u32, (SetValue, SystemTime)> = HashMap::new();

  mqtt.publish(online_msg).await?;
  while let Some(evt) = events.next().await {
    match evt {
      ApplicationMessage::DeviceUpdate(device, update) => {
        info!("{} updated with {:?}", device.name(), update);

        match device {
          Device::Binary(device) => match update {
            DeviceUpdate::State(state) => {
              let topic = format!("enet/instance-1/{}/state", device.number());
              let payload = match state {
                DeviceState::On => "ON",
                DeviceState::Off => "OFF",
              };
              let state_msg = Message::new_retained(topic, payload, 2);
              mqtt.publish(state_msg).await?;
            }
            DeviceUpdate::Brightness(_) => unreachable!(),
          },

          Device::Dimmer(device) => match update {
            DeviceUpdate::State(state) => {
              let topic = format!("enet/instance-1/{}/state", device.number());
              let payload = match state {
                DeviceState::On => "ON",
                DeviceState::Off => "OFF",
              };
              let state_msg = Message::new_retained(topic, payload, 2);
              mqtt.publish(state_msg).await?;
            }
            DeviceUpdate::Brightness(brightness) => {
              let topic = format!("enet/instance-1/{}/bri", device.number());
              let value_s = brightness.get().to_string();
              let brightness_msg = Message::new_retained(topic, value_s, 2);
              mqtt.publish(brightness_msg).await?;
            }
          },
        }
      }

      ApplicationMessage::MqttMessage(msg) => {
        info!(msg.topic = msg.topic(), "MQTT message received.");

        if let Some(msg) = MqttMessage::try_match(msg) {
          match msg {
            MqttMessage::SetDeviceState(number, value) => {
              info!("Set device state command received.");
              if client.device(number).is_none() {
                warn!("Got set-value command for unknown device number {}", number);
                continue;
              };

              let value = if let Some((bri_value, then)) = bris.remove(&number) {
                // if more than 15 seconds has passed since we got the "set brightnes" command,
                // we ignore it.
                let elapsed = then.elapsed().unwrap();
                if elapsed > Duration::from_secs(15) {
                  value.into_device_value()
                } else {
                  bri_value
                }
              } else {
                value.into_device_value()
              };

              if let Err(e) = client.set_value(number, value).await {
                warn!("Failed to set value: {:?}", e);
              }
            }

            MqttMessage::SetDeviceBrightness(number, value) => {
              info!("Set device brightness command received.");

              if client.device(number).is_none() {
                warn!(
                  "Got set-brightness command for unknown device number {}",
                  number
                );
                continue;
              };

              bris.insert(number, (value.into_device_value(), SystemTime::now()));
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

enum ApplicationMessage {
  DeviceUpdate(Device, DeviceUpdate),
  MqttMessage(Message),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum DeviceUpdate {
  State(DeviceState),
  Brightness(DeviceBrightness),
}

impl DeviceUpdate {
  fn state(state: DeviceState) -> Ready<Self> {
    ready(Self::State(state))
  }

  fn brightness(brightness: DeviceBrightness) -> Ready<Self> {
    ready(Self::Brightness(brightness))
  }
}

impl From<(Device, DeviceUpdate)> for ApplicationMessage {
  #[inline]
  fn from((device, value): (Device, DeviceUpdate)) -> Self {
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

fn msg_retained(topic: impl Into<String>, data: &serde_json::Value) -> Message {
  let bytes = serde_json::to_vec(data).unwrap();
  Message::new_retained(topic, bytes, 2)
}
