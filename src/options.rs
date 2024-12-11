use super::{ConnectorOptions, IntifaceCLIErrorEnum, IntifaceError};

use super::frontend::{self, FrontendPBufChannel};
use argh::FromArgs;
#[cfg(target_os = "windows")]
use buttplug::server::comm_managers::xinput::XInputDeviceCommunicationManagerBuilder;
use buttplug::server::{
  comm_managers::{
    btleplug::BtlePlugCommunicationManagerBuilder,
    lovense_connect_service::LovenseConnectServiceCommunicationManagerBuilder,
    lovense_dongle::{
      LovenseHIDDongleCommunicationManagerBuilder, LovenseSerialDongleCommunicationManagerBuilder,
    },
    serialport::SerialPortCommunicationManagerBuilder,
    websocket_server::websocket_server_comm_manager::WebsocketServerDeviceCommunicationManagerBuilder,
    DeviceCommunicationManagerBuilder,
  },
  ButtplugRemoteServer,
};

use std::fs;
use tokio_util::sync::CancellationToken;
use tracing::Level;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub fn check_log_level() -> Option<Level> {
  let args: IntifaceCLIArguments = argh::from_env();
  args.log
}

pub fn check_frontend_pipe(token: CancellationToken) -> Option<FrontendPBufChannel> {
  let args: IntifaceCLIArguments = argh::from_env();
  if args.frontendpipe {
    Some(frontend::run_frontend_task(token))
  } else {
    None
  }
}

pub fn parse_options() -> Result<Option<ConnectorOptions>, IntifaceCLIErrorEnum> {
  let args: IntifaceCLIArguments = argh::from_env();

  // Options that will do a thing then exit:
  //
  // - serverversion
  // - generatecert
  if args.serverversion || args.version {
    debug!("Server version command sent, printing and exiting.");
    println!(
      "Intiface CLI (Rust Edition) Version {}, Commit {}, Built {}",
      VERSION,
      env!("VERGEN_GIT_SHA_SHORT"),
      env!("VERGEN_BUILD_TIMESTAMP")
    );
    return Ok(None);
  }

  // Options that set up the server networking

  let mut connector_info = ConnectorOptions::default();
  let mut connector_info_set = false;

  if args.wsallinterfaces {
    info!("Intiface CLI Options: Websocket Use All Interfaces option passed.");
    connector_info.ws_listen_on_all_interfaces = true;
    connector_info_set = true;
  }

  if let Some(wsinsecureport) = &args.wsinsecureport {
    info!(
      "Intiface CLI Options: Websocket Insecure Port {}",
      wsinsecureport
    );
    connector_info.ws_insecure_port = Some(*wsinsecureport);
    connector_info_set = true;
  }

  if let Some(ipcpipe) = &args.ipcpipe {
    // TODO We should actually implement pipes :(
    info!("Intiface CLI Options: IPC Pipe Name {}", ipcpipe);
  }

  // If we don't have a device configuration by this point, panic.

  if !connector_info_set {
    return Err(
      IntifaceError::new(
        "Must have a connection argument (wsinsecureport, wssecureport, ipcport) to run!",
      )
      .into(),
    );
  }

  connector_info
    .server_builder
    .name(&args.servername)
    .max_ping_time(args.pingtime)
    .allow_raw_messages(args.allowraw);

  if args.frontendpipe {
    info!("Intiface CLI Options: Using frontend pipe");
    connector_info.use_frontend_pipe = true;
  }

  if args.stayopen {
    info!("Intiface CLI Options: Leave server open after disconnect.");
    connector_info.stay_open = true;
  }

  // Options that set up Buttplug server parameters

  if let Some(deviceconfig) = &args.deviceconfig {
    info!(
      "Intiface CLI Options: External Device Config {}",
      deviceconfig
    );
    match fs::read_to_string(deviceconfig) {
      Ok(cfg) => connector_info
        .server_builder
        .device_configuration_json(Some(cfg)),
      Err(err) => panic!("Error opening external device configuration: {:?}", err),
    };
  }

  if let Some(userdeviceconfig) = &args.userdeviceconfig {
    info!(
      "Intiface CLI Options: User Device Config {}",
      userdeviceconfig
    );
    match fs::read_to_string(userdeviceconfig) {
      Ok(cfg) => connector_info
        .server_builder
        .user_device_configuration_json(Some(cfg)),
      Err(err) => panic!("Error opening user device configuration: {:?}", err),
    };
  }

  Ok(Some(connector_info))
}
