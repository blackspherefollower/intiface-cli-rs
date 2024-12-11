use argh::FromArgs;
use intiface_engine::{EngineOptions, EngineOptionsExternal, IntifaceEngine, IntifaceEngineError};
use tokio::{select, signal::ctrl_c};
use tracing::{debug, info, Level};
use tracing_subscriber::{
  filter::{EnvFilter, LevelFilter},
  layer::SubscriberExt,
  util::SubscriberInitExt,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");


/// command line interface for intiface/buttplug.
///
/// Note: Commands are one word to keep compat with C#/JS executables currently.
#[derive(FromArgs)]
struct IntifaceCLIArguments {
  // Options that do something then exit
  /// print version and exit.
  #[argh(switch)]
  version: bool,

  /// print version and exit.
  #[argh(switch)]
  serverversion: bool,

  // Options that set up the server networking
  /// if passed, websocket server listens on all interfaces. Otherwise, only
  /// listen on 127.0.0.1.
  #[argh(switch)]
  wsallinterfaces: bool,

  /// insecure port for websocket servers.
  #[argh(option)]
  wsinsecureport: Option<u16>,

  /// pipe name for ipc server
  #[argh(option)]
  ipcpipe: Option<String>,

  // Options that set up communications with intiface GUI
  /// if passed, output protobufs for parent process via stdio, instead of strings.
  #[argh(switch)]
  frontendpipe: bool,

  // Options that set up Buttplug server parameters
  /// name of server to pass to connecting clients.
  #[argh(option)]
  #[argh(default = "\"Buttplug Server\".to_owned()")]
  servername: String,

  /// path to the device configuration file
  #[argh(option)]
  deviceconfig: Option<String>,

  /// path to user device configuration file
  #[argh(option)]
  userdeviceconfig: Option<String>,

  /// ping timeout maximum for server (in milliseconds)
  #[argh(option)]
  #[argh(default = "0")]
  pingtime: u32,

  /// if passed, server will stay running after client disconnection
  #[argh(switch)]
  stayopen: bool,

  /// set log level for output
  #[allow(dead_code)]
  #[argh(option)]
  log: Option<Level>,

  /// allow raw messages (dangerous, only use for development)
  #[argh(switch)]
  allowraw: bool,

  /// turn off bluetooth le device support
  #[argh(switch)]
  without_bluetooth_le: bool,

  /// turn off serial device support
  #[argh(switch)]
  without_serial: bool,

  /// turn off hid device support
  #[allow(dead_code)]
  #[argh(switch)]
  without_hid: bool,

  /// turn off lovense dongle serial device support
  #[argh(switch)]
  without_lovense_dongle_serial: bool,

  /// turn off lovense dongle hid device support
  #[argh(switch)]
  without_lovense_dongle_hid: bool,

  /// turn off xinput gamepad device support (windows only)
  #[argh(switch)]
  without_xinput: bool,

  /// turn on lovense connect app device support (off by default)
  #[argh(switch)]
  with_lovense_connect: bool,

  /// turn on websocket server device comm manager
  #[argh(switch)]
  with_websocket_server_device: bool,
}

pub fn setup_console_logging(log_level: Option<Level>) {
  if log_level.is_some() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(LevelFilter::from(log_level))
        .try_init()
        .unwrap();
  } else {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
          EnvFilter::try_from_default_env()
              .or_else(|_| EnvFilter::try_new("info"))
              .unwrap(),
        )
        .try_init()
        .unwrap();
  };
  println!("Intiface Server, starting up with stdout output.");
}

#[tokio::main(flavor = "current_thread")] //#[tokio::main]
async fn main() -> Result<(), IntifaceEngineError> {
  let args: IntifaceCLIArguments = argh::from_env();

  if args.version {
    debug!("Server version command sent, printing and exiting.");
    println!(
      "Intiface CLI (Repeater) Version {}, Commit {}, Built {}",
      VERSION,
      option_env!("VERGEN_GIT_SHA_SHORT").unwrap_or("unknown"),
      option_env!("VERGEN_BUILD_TIMESTAMP").unwrap_or("unknown")
    );
    return Ok(());
  }

  let mut options = EngineOptionsExternal::default();
  if let Some(port) = args.wsinsecureport {
    options.repeater_mode = true;
    options.repeater_local_port = Some(port);
    options.repeater_remote_address = Some("ws://localhost:12345".to_string());
  }

  setup_console_logging(args.log);

  let engine = IntifaceEngine::default();
  let eopts = EngineOptions::from(options);
  select! {
    result = engine.run(&eopts, None, &None) => {
      if let Err(e) = result {
        println!("Server errored while running:");
        println!("{:?}", e);
      }
    }
    _ = ctrl_c() => {
      info!("Control-c hit, exiting.");
      engine.stop();
    }
  }

  Ok(())
}
