use clap::{AppSettings, ArgSettings, Clap};
use color_eyre::{eyre::Context, Result};
use paho_mqtt::{ConnectOptions, ConnectOptionsBuilder};
use tokio::net::lookup_host;
use tracing::{event, Level};

#[derive(Clap, Debug, PartialEq, Clone, Copy)]
pub enum LogFormat {
  Pretty,
  Json,
}

#[derive(Debug, Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Args {
  /// Log output format.
  #[clap(
    arg_enum,
    long = "log-format",
    short = 'f',
    env = "LOG_FORMAT",
    default_value = "pretty"
  )]
  pub log_format: LogFormat,

  /// eNet gateway address (example: 1.1.1.1:9050).
  #[clap(long = "gateway", short = 'g', env = "ENET_GATEWAY")]
  pub gateway: String,

  /// MQTT args.
  #[clap(flatten)]
  pub mqtt: Mqtt,
}

#[derive(Debug, Clap)]
pub struct Mqtt {
  /// MQTT broker address (example: mosquitto.org).
  #[clap(long = "mqtt-host", short = 'h', env = "ENET_MQTT_HOST")]
  pub host: String,

  /// MQTT port.
  #[clap(
    long = "mqtt-port",
    short = 'p',
    env = "ENET_MQTT_PORT",
    default_value = "1883"
  )]
  pub port: u16,

  /// MQTT auth.
  #[clap(flatten)]
  pub auth: MqttAuth,
}

impl Mqtt {
  pub async fn into_connect_options(self) -> Result<ConnectOptions> {
    let Self { host, port, auth } = self;
    let hosts = lookup_host((&*host, port))
      .await
      .wrap_err_with(|| format!("Failed to resolve host '{}'", host))?;

    let mut builder = ConnectOptionsBuilder::new();

    let mut uris = Vec::with_capacity(4);
    for host in hosts {
      let uri = format!("tcp://{}", host);
      uris.push(uri);
    }

    builder.server_uris(&*uris);
    if let Some(username) = auth.username {
      builder.user_name(username);
    }

    if let Some(password) = auth.password {
      builder.password(password);
    }

    event!(
      Level::INFO,
      mqtt.uris.len = uris.len(),
      mqtt.uris = ?uris,
      "Creating mqtt connect options",
    );

    Ok(builder.finalize())
  }
}

#[derive(Debug, Clap)]
pub struct MqttAuth {
  /// MQTT username.
  #[clap(long = "mqtt-user", short = 'u', env = "ENET_MQTT_USER")]
  pub username: Option<String>,

  /// MQTT password.
  #[clap(
    long = "mqtt-pass",
    short = 's',
    env = "ENET_MQTT_PASS",
    requires = "username",
    setting = ArgSettings::HideEnvValues
  )]
  pub password: Option<String>,
}
