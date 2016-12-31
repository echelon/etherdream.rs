// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// Etherdream.rs is a library for the EtherDream laser projector DAC.
// Some of the documentation text is taken directly from the Etherdream
// website, and the copyright belongs to Jacob Potter.
// See http://ether-dream.com/protocol.html

//! This module describes the EtherDream protocol.

use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use byteorder::WriteBytesExt;
use error::EtherdreamError;
use std::io::Cursor;

/// The highest value that can be specified for a single color channel.
pub const COLOR_MAX : u16 = 65535;
/// The lowest value that can be specified for a single color channel.
pub const COLOR_MIN : u16 = 0;
/// The highest x coordinate.
pub const X_MAX : i16 = 32767;
/// The lowest x coordinate.
pub const X_MIN : i16 = -32768;
/// The highest y coordinate.
pub const Y_MAX : i16 = 32767;
/// The lowest y coordinate.
pub const Y_MIN : i16 = -32768;

/// Byte designating the 'begin' command.
pub const COMMAND_BEGIN : u8   = 0x62;
/// Byte designating the 'data' command.
pub const COMMAND_DATA : u8    = 0x64;
/// Byte designating the 'ping' command.
pub const COMMAND_PING : u8    = 0x3F;
/// Byte designating the 'prepare' command.
pub const COMMAND_PREPARE : u8 = 0x70;

/// Ack byte
pub const RESPONSE_ACK: u8         = 0x61;
/// Nack byte - buffer full
pub const RESPONSE_BUFFER_FULL: u8 = 0x46;
/// Nack byte - invalid command
pub const RESPONSE_INVALID_CMD: u8 = 0x49;
/// Nack byte - stop
pub const RESPONSE_STOP: u8        = 0x21;

/** A 22-byte response the DAC sends to any command. */
#[derive(Clone, Copy, Debug)]
pub struct DacResponse {
  /// One byte ACK/NACK.
  pub acknowledgement: AckCode,

  /// One byte repeat of the command that was received.
  /// For sanity checking.
  pub command : CommandCode,

  /// 20-byte status of the dac.
  pub status : DacStatus,
}

impl DacResponse {
  /// Parse a DacResponse from a 22 byte body.
  pub fn parse(bytes: &[u8]) -> Result<DacResponse, EtherdreamError> {
    if bytes.len() != 22 {
      return Err(EtherdreamError::InvalidResponseLength {
        description: format!("Response is {} bytes, not the expected 22 bytes.",
            bytes.len()),
      });
    }

    let status = DacStatus::parse(&bytes[2..])?;

    Ok(DacResponse {
      acknowledgement: AckCode::parse(bytes[0]),
      command: CommandCode::parse(bytes[1]),
      status: status,
    })
  }

  /// Whether or not the response is a successful ACK.
  pub fn is_ack(&self) -> bool {
    self.acknowledgement.is_ack()
  }
}

/// Ack or Nack Responses.
#[derive(Clone, Copy, Debug)]
pub enum AckCode {
  Ack,
  NackBufferFull,
  NackInvalid,
  NackStop,
  NackUnknown { code: u8 },
}

/// EtherDream Commands.
#[derive(Clone, Copy, Debug)]
pub enum CommandCode {
  Begin,
  Data,
  Ping,
  Prepare,
  CommandUnknown { code: u8 },
  // TODO: More.
}

impl AckCode {
  pub fn parse(byte: u8) -> AckCode {
    match byte {
      RESPONSE_ACK => AckCode::Ack,
      RESPONSE_BUFFER_FULL => AckCode::NackBufferFull,
      RESPONSE_INVALID_CMD => AckCode::NackInvalid,
      RESPONSE_STOP => AckCode::NackStop,
      _ => AckCode::NackUnknown { code: byte },
    }
  }

  /// Whether or not the code is a successful ACK.
  pub fn is_ack(&self) -> bool {
    match *self {
      AckCode::Ack => true,
      _ => false,
    }
  }
}

impl CommandCode {
  pub fn parse(byte: u8) -> CommandCode {
    match byte {
      COMMAND_BEGIN => CommandCode::Begin,
      COMMAND_DATA => CommandCode::Data,
      COMMAND_PING => CommandCode::Ping,
      COMMAND_PREPARE => CommandCode::Prepare,
      _ => CommandCode::CommandUnknown { code: byte },
    }
  }
}

/** The DAC periodically sends state information. */
#[derive(Clone, Copy, Debug)]
pub struct DacStatus {
  pub protocol: u8,

  /**
   * The light engine is one of three state machines in the DAC.
   *
   * The states are:
   *
   *  - 0: Ready.
   *  - 1: Warmup. In the case where the DAC is also used for thermal
   *       control of laser apparatus, this is the state that is
   *       entered after power-up.
   *  - 2: Cooldown. Lasers are off but thermal control is still active
   *  - 3: Emergency stop. An emergency stop has been triggered, either
   *       by an E-stop input on the DAC, an E-stop command over the
   *       network, or a fault such as over-temperature.
   *
   *  (Since thermal control is not implemented yet, it is not defined
   *  how transitions to and from the "Warmup" and "Cooldown" states
   *  occur.)
   */
  pub light_engine_state: u8,

  /**
   * The playback_state is one of three state machines in the DAC.
   * It reports the state of the playback system.
   *
   * The DAC has one playback system, which buffers data and sends it
   * to the analog output hardware at its current point rate. At any
   * given time, the playback system is connected to a source. Usually,
   * the source is the network streamer, which uses the protocol
   * described in this document; however, other sources exist, such as
   * a built-in abstract generator and file playback from SD card. The
   * playback system is in one of the following states:
   *
   *   - 0: Idle. This is the default state. No points may be added to
   *        the buffer. No output is generated; all analog outputs are
   *        at 0v, and the shutter is controlled by the data source.
   *   - 1: Prepared. The buffer will accept points. The output is the
   *        same as in the Idle state.
   *   - 2: Playing. Points are being sent to the output.
   *
   * See playback_flags for additional information.
   */
  pub playback_state: u8,

  /**
   * The currently-selected data source is specified in the source field:
   *
   *   - 0: Network streaming (the protocol defined in the rest of this
   *        document).
   *   - 1: ILDA playback from SD card.
   *   - 2: Internal abstract generator.
   */
  pub source: u8,

  /**
   * The light_engine_state field gives the current state of the light
   * engine. If the light engine is Ready, light_engine_flags will be 0.
   * Otherwise, bits in light_engine_flags will be set as follows:
   *
   * [0]: Emergency stop occurred due to E-Stop packet or invalid
   *      command.
   * [1]: Emergency stop occurred due to E-Stop input to projector.
   * [2]: Emergency stop input to projector is currently active.
   * [3]: Emergency stop occurred due to overtemperature condition.
   * [4]: Overtemperature condition is currently active.
   * [5]: Emergency stop occurred due to loss of Ethernet link.
   * [15:5]: Future use.
   */
  pub light_engine_flags: u16,

  /**
   * The playback_flags field may be nonzero during normal operation.
   * Its bits are defined as follows:
   *
   * [0]: Shutter state: 0 = closed, 1 = open.
   * [1]: Underflow. 1 if the last stream ended with underflow, rather
   *      than a Stop command. Reset to zero by the Prepare command.
   * [2]: E-Stop. 1 if the last stream ended because the E-Stop state
   *      was entered. Reset to zero by the Prepare command.
   */
  pub playback_flags: u16,

  /// TODO: Undocumented?
  pub source_flags: u16,

  /** Reports the number of points currently buffered. */
  pub buffer_fullness: u16,

  /**
   * The number of points per second for which the DAC is configured
   * (if Prepared or Playing), or zero if the DAC is idle.
   */
  pub point_rate: u32,

  /**
   * The number of points that the DAC has actually emitted since it
   * started playing (if Playing), or zero (if Prepared or Idle).
   */
  pub point_count: u32,
}

impl DacStatus {
  /// Parse a DacStatus from raw bytes. DacStatuses are 20 bytes.
  pub fn parse(bytes: &[u8]) -> Result<DacStatus, EtherdreamError> {
    if bytes.len() < 20 {
      return Err(EtherdreamError::InvalidResponseLength {
        description: format!("Response is {} bytes; must be no fewer than 20.",
          bytes.len()),
      });
    }

    let mut reader = Cursor::new(&bytes[4..20]);

    Ok(DacStatus {
      protocol           : bytes[0],
      light_engine_state : bytes[1],
      playback_state     : bytes[2],
      source             : bytes[3],
      light_engine_flags : reader.read_u16::<LittleEndian>()?,
      playback_flags     : reader.read_u16::<LittleEndian>()?,
      source_flags       : reader.read_u16::<LittleEndian>()?,
      buffer_fullness    : reader.read_u16::<LittleEndian>()?,
      point_rate         : reader.read_u32::<LittleEndian>()?,
      point_count        : reader.read_u32::<LittleEndian>()?,
    })
  }

  // TODO: Unsafe; remove?
  pub fn serialize(&self) -> Vec<u8> {
    let mut v = Vec::new();
    v.push(self.protocol);
    v.push(self.light_engine_state);
    v.push(self.playback_state);
    v.push(self.source);
    v.write_u16::<LittleEndian>(self.light_engine_flags).unwrap();
    v.write_u16::<LittleEndian>(self.playback_flags).unwrap();
    v.write_u16::<LittleEndian>(self.source_flags).unwrap();
    v.write_u16::<LittleEndian>(self.buffer_fullness).unwrap();
    v.write_u32::<LittleEndian>(self.point_rate).unwrap();
    v.write_u32::<LittleEndian>(self.point_count).unwrap();
    v
  }
}

/** MAC address reported by the DAC. */
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MacAddress {
  pub address: [u8; 6]
}

impl MacAddress {
  /// Parse a MacAddress from raw bytes. MacAddresses are 6 bytes.
  pub fn parse(bytes: &[u8]) -> Result<MacAddress, EtherdreamError> {
    if bytes.len() < 6 {
      return Err(EtherdreamError::InvalidResponseLength {
        description: format!("MacAddress is {} bytes; must be no fewer than 6.",
          bytes.len()),
      });
    }

    Ok(MacAddress {
      address: [
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
      ],
    })
  }
}

// 6 bytes (MacAddress) + 10 bytes + 20 bytes (DacStatus) = 36 bytes
/** The DAC periodically sends state information. */
#[derive(Clone, Copy, Debug)]
pub struct Broadcast {
  pub mac_address : MacAddress,
  pub hw_revision : u16,
  pub sw_revision : u16,
  pub buffer_capacity : u16,
  pub max_point_rate : u32,
  pub status : DacStatus,
}

impl Broadcast {
  /// Parse a Broadcast from raw bytes. Broadcasts are 36 bytes.
  pub fn parse(bytes: &[u8]) -> Result<Broadcast, EtherdreamError> {
    if bytes.len() < 36 {
      return Err(EtherdreamError::InvalidResponseLength {
        description: format!("Broadcast is {} bytes; must be no fewer than 36.",
          bytes.len()),
      });
    }

    let mut reader = Cursor::new(&bytes[6..32]);
    Ok(Broadcast {
      mac_address     : MacAddress::parse(bytes)?,
      hw_revision     : reader.read_u16::<LittleEndian>()?,
      sw_revision     : reader.read_u16::<LittleEndian>()?,
      buffer_capacity : reader.read_u16::<LittleEndian>()?,
      max_point_rate  : reader.read_u32::<LittleEndian>()?,
      status          : DacStatus::parse(&bytes[16..36])?,
    })
  }

  // TODO: Unsafe; remove?
  pub fn serialize(&self) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend(&self.mac_address.address);
    v.write_u16::<LittleEndian>(self.hw_revision).unwrap();
    v.write_u16::<LittleEndian>(self.sw_revision).unwrap();
    v.write_u16::<LittleEndian>(self.buffer_capacity).unwrap();
    v.write_u32::<LittleEndian>(self.max_point_rate).unwrap();
    v.extend(self.status.serialize());
    v
  }
}

// TODO: Docs, tests.
/** Begin command. */
#[derive(Clone, Copy, Debug)]
pub struct Begin {
  /// Unused.
  pub low_water_mark: u16,
  /// Point Rate.
  pub point_rate : u32,
}

impl Begin {
  pub fn serialize(&self) -> Vec<u8> {
    let mut v = Vec::new();
    v.push(COMMAND_BEGIN); // 'b'
    v.write_u16::<LittleEndian>(self.low_water_mark).unwrap();
    v.write_u32::<LittleEndian>(self.point_rate).unwrap();
    v
  }
}

// TODO: Docs, tests.
/** 18-byte point data for a single point. */
#[derive(Clone, Copy, Debug)]
pub struct Point {
  pub control: u16,
  pub x: i16,
  pub y: i16,
  pub r: u16,
  pub g: u16,
  pub b: u16,
  pub i: u16,
  pub u1: u16,
  pub u2: u16,
}

impl Point {
  /// Point CTOR.
  pub fn xy_rgb(x: i16, y: i16, r: u16, g: u16, b: u16) -> Point {
    Point {
      control: 0,
      x: x,
      y: y,
      r: r,
      g: g,
      b: b,
      i: 0,
      u1: 0,
      u2: 0,
    }
  }

  /// Point CTOR.
  /// Use the same intensity value for all color channels.
  pub fn xy_luma(x: i16, y: i16, luminance: u16) -> Point {
    Point {
      control: 0,
      x: x,
      y: y,
      r: luminance,
      g: luminance,
      b: luminance,
      i: luminance,
      u1: 0,
      u2: 0,
    }
  }

  /// Blank point CTOR.
  pub fn xy_blank(x: i16, y: i16) -> Point {
    Point::xy_luma(x, y, 0)
  }

  /// Point CTOR.
  /// If set to on, the lasers are at full power. Otherwise, they're off.
  pub fn xy_binary(x: i16, y: i16, on: bool) -> Point {
    let c = if on { COLOR_MAX } else { 0 };
    Point::xy_rgb(x, y, c, c, c)
  }

  pub fn serialize(&self) -> Vec<u8> {
    // NB: Website documentation is incorrect about byte order: the "rgb" color
    // channels each come before "i".
    let mut v = Vec::new();
    v.write_u16::<LittleEndian>(self.control).unwrap();
    v.write_i16::<LittleEndian>(self.x).unwrap();
    v.write_i16::<LittleEndian>(self.y).unwrap();
    v.write_u16::<LittleEndian>(self.r).unwrap();
    v.write_u16::<LittleEndian>(self.g).unwrap();
    v.write_u16::<LittleEndian>(self.b).unwrap();
    v.write_u16::<LittleEndian>(self.i).unwrap();
    v.write_u16::<LittleEndian>(self.u1).unwrap();
    v.write_u16::<LittleEndian>(self.u2).unwrap();
    v
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_dac_status_parse() {
    let buf = vec![
      0,
      100,
      200,
      255,
      255, 0,
      0, 1,
      1, 1,
      255, 255,
      255, 1, 255, 1,
      255, 255, 255, 255,
    ];

    let status = DacStatus::parse(&buf).unwrap();
    assert_eq!(0, status.protocol);
    assert_eq!(100, status.light_engine_state);
    assert_eq!(200, status.playback_state);
    assert_eq!(255, status.source);

    assert_eq!(255, status.light_engine_flags);
    assert_eq!(256, status.playback_flags);
    assert_eq!(257, status.source_flags);
    assert_eq!(65535, status.buffer_fullness);

    assert_eq!(33489407, status.point_rate);
    assert_eq!(4294967295, status.point_count);
  }

  #[test]
  fn test_mac_address_parse() {
    let buf = vec![1, 2, 3, 4, 5, 6];

    let address = MacAddress::parse(&buf).unwrap();
    assert_eq!(1, address.address[0]);
    assert_eq!(2, address.address[1]);
    assert_eq!(3, address.address[2]);
    assert_eq!(4, address.address[3]);
    assert_eq!(5, address.address[4]);
    assert_eq!(6, address.address[5]);
  }

  #[test]
  fn test_broadcast_parse() {
    let buf = vec![
      // Address
      0, 1, 2, 3, 4, 5,
      // Broadcast
      0, 255,
      255, 0,
      1, 2,
      1, 2, 3, 4,
      // Status
      0,
      100,
      200,
      255,
      255, 0,
      0, 1,
      1, 1,
      255, 255,
      255, 1, 255, 1,
      255, 255, 255, 255,
    ];

    let broadcast = Broadcast::parse(&buf).unwrap();
    assert_eq!(MacAddress { address: [0, 1, 2, 3, 4, 5] }, broadcast.mac_address);
    assert_eq!(65280, broadcast.hw_revision);
    assert_eq!(255, broadcast.sw_revision);
    assert_eq!(513, broadcast.buffer_capacity);
    assert_eq!(67305985, broadcast.max_point_rate);
  }

  #[test]
  fn test_point_xy_rgb() {
    let point = Point::xy_rgb(10_000, -10_000, 32, 128, 1028);
    assert_eq!(10_000, point.x);
    assert_eq!(-10_000, point.y);
    assert_eq!(32, point.r);
    assert_eq!(128, point.g);
    assert_eq!(1028, point.b);
    assert_eq!(0, point.i);
  }

  #[test]
  fn test_point_xy_luma() {
    let point = Point::xy_luma(5, -5, 127);
    assert_eq!(5, point.x);
    assert_eq!(-5, point.y);
    assert_eq!(127, point.r);
    assert_eq!(127, point.g);
    assert_eq!(127, point.b);
    assert_eq!(127, point.i);
  }

  #[test]
  fn test_point_xy_blank() {
    let point = Point::xy_blank(10, -20);
    assert_eq!(10, point.x);
    assert_eq!(-20, point.y);
    assert_eq!(0, point.r);
    assert_eq!(0, point.g);
    assert_eq!(0, point.b);
    assert_eq!(0, point.i);
  }

  #[test]
  fn test_point_xy_binary() {
    let point = Point::xy_binary(20_000, -20_000, true);
    assert_eq!(20_000, point.x);
    assert_eq!(-20_000, point.y);
    assert_eq!(65535, point.r);
    assert_eq!(65535, point.g);
    assert_eq!(65535, point.b);

    let point = Point::xy_binary(1000, 2000, false);
    assert_eq!(1000, point.x);
    assert_eq!(2000, point.y);
    assert_eq!(0, point.r);
    assert_eq!(0, point.g);
    assert_eq!(0, point.b);
  }
}

