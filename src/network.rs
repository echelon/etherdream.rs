// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// Etherdream.rs, a library for the EtherDream laser projector DAC.

use net2::UdpBuilder;
use std::io::Error;
use std::net::IpAddr;

use protocol::Broadcast;

/// The primary port for communications with the EtherDream.
pub const COMMUNICATION_PORT : u16 = 7765;

/// The port EtherDream broadcasts its availability on.
pub const BROADCAST_PORT : u16 = 7654;

/// A DAC found from listening to UDP broadcasts.
#[derive(Clone, Copy, Debug)]
pub struct SearchResult {
  /// IP address of the DAC.
  pub ip_address : IpAddr,

  /// The Broadcast that was sent.
  pub broadcast : Broadcast,
}

// TODO: Unsafe
/// Blocking function that will return the first EtherDream DAC it finds
/// via listening for UDP broadcasts.
pub fn find_first_dac() -> Result<SearchResult, Error> {
  let udp = UdpBuilder::new_v4()?;
  udp.reuse_address(true)?;

  let socket = udp.bind(("0.0.0.0", BROADCAST_PORT))?;

  let mut buf = [0u8; 128];
  let result = socket.recv_from(&mut buf)?;

  let broadcast = Broadcast::parse(&buf[0..36])?;

  Ok(SearchResult {
    ip_address : result.1.ip(),
    broadcast: broadcast,
  })
}

