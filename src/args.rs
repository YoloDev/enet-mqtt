use clap::{AppSettings, Clap};

#[derive(Clap, Debug, PartialEq, Clone, Copy)]
pub enum LogFormat {
  Pretty,
  Json,
}

#[derive(Debug, Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Args {
  /// Log output format
  #[clap(
    arg_enum,
    long = "log-format",
    short = 'f',
    env = "LOG_FORMAT",
    default_value = "pretty"
  )]
  pub log_format: LogFormat,

  /// eNet gateway address (example: 1.1.1.1:9050)
  #[clap(long = "gateway", short = 'g', env = "ENET_GATEWAY")]
  pub gateway: String,

  /// MQTT broker address (example: mqtt://mosquitto.org:1883)
  #[clap(long = "mqtt-broker", short = 'b', env = "ENET_MQTT_BROKER")]
  pub mqtt_broker: String,
}
