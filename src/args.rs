use clap::{AppSettings, ArgSettings, Clap};
use paho_mqtt::{ConnectOptions, ConnectOptionsBuilder};
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
  pub port: u32,

  /// MQTT auth.
  #[clap(flatten)]
  pub auth: MqttAuth,
}

impl From<Mqtt> for ConnectOptions {
  fn from(val: Mqtt) -> Self {
    let mut builder = ConnectOptionsBuilder::new();
    let uri = format!("tcp://{}:{}", val.host, val.port);

    builder.server_uris(&[&*uri]);
    if let Some(username) = val.auth.username {
      builder.user_name(username);
    }

    if let Some(password) = val.auth.password {
      builder.password(password);
    }

    event!(
      Level::INFO,
      mqtt.uri = &*uri,
      "creating mqtt connect options",
    );
    builder.finalize()
  }
}

impl From<Mqtt> for Option<ConnectOptions> {
  fn from(val: Mqtt) -> Self {
    Some(val.into())
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
