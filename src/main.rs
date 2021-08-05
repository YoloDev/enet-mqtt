mod args;
mod evlist;

use args::{Args, LogFormat};
use clap::Clap;
use color_eyre::{eyre::Context, Report};
use enet_client::{
  dev::{DeviceKind, OnValue},
  EnetClient,
};
use evlist::EvList;
use futures::StreamExt;
use paho_mqtt::{AsyncClient, CreateOptions, Message};
use serde_json::json;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Report> {
  let args = Args::parse();
  setup(args.log_format)?;

  info!("Connecting to eNet gateway {}.", args.gateway);
  let client = EnetClient::new(args.gateway).await?;
  info!("eNet client ready.");

  info!(
    "Connecting to MQTT broker {}:{} with user '{}'",
    args.mqtt.host,
    args.mqtt.port,
    args.mqtt.auth.username.as_deref().unwrap_or("<no user>")
  );
  let mqtt = AsyncClient::new(CreateOptions::new()).wrap_err("Failed to create mqtt client.")?;
  mqtt
    .connect(args.mqtt)
    .await
    .wrap_err("Failed to connect to mqtt.")?;

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
          "brightness": false
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
          "brightness": true
        });

        let bytes = serde_json::to_vec(&config).unwrap();
        Message::new_retained(topic, bytes, 2)
      }
      DeviceKind::Blinds => continue,
    };

    mqtt.publish(config_msg).await?;
  }

  let mut stream = EvList::new(subscriptions);
  while let Some((device, value)) = stream.next().await {
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
