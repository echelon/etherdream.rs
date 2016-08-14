// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>

use std::f64;
use std::f64::consts::PI;
use byteorder::LittleEndian;
use byteorder::WriteBytesExt;
use protocol::Begin;
use protocol::COMMAND_BEGIN;
use protocol::COMMAND_DATA;
use protocol::COMMAND_PREPARE;
use protocol::Point;
use std::io::Read;
use std::io::Write;
use std::net::IpAddr;
use std::net::TcpStream;

pub struct Dac {
  ip_address: IpAddr,
  stream: TcpStream,
}

impl Dac {

  /// CTOR.
  pub fn new(ip_address: IpAddr) -> Dac {
    let stream = TcpStream::connect((ip_address, 7765u16)).unwrap(); // FIXME
    Dac {
      ip_address: ip_address,
      stream: stream,
    }
  }

  // TODO TEMPORARY.
  pub fn play_demo(&mut self) {
    println!("Send begin");
    self.begin();

    println!("Send prepare");
    self.prepare();

    loop {
      println!("Send data");
      self.write_data();
    }
  }

  fn prepare(&mut self) {
    let cmd = [ COMMAND_PREPARE ];
    self.stream.write(&cmd).unwrap(); // FIXME

    let mut buf = [0; 2048];
    self.stream.read(&mut buf).unwrap(); // FIXME
  }

  fn begin(&mut self) {
    let cmd = Begin { low_water_mark: 0, point_rate: 30_000 };
    self.stream.write(&cmd.serialize()).unwrap(); // FIXME
    let mut buf = [0; 2048];
    self.stream.read(&mut buf).unwrap(); // FIXME
  }

  fn write_data(&mut self) {
    let num_points = 100; // TODO

    let mut cmd = Vec::new();
    cmd.push(COMMAND_DATA);
    cmd.write_u16::<LittleEndian>(num_points).unwrap();

    // TODO WRITE POINTS
    for i in 0 .. num_points {
      //let j = ((i * 1.0f64) / num_points) * 2 * PI;
      let pt = Point::xy_rgb(i as i16, i as i16, 255, 255, 255);
      cmd.extend(pt.serialize());
    }

    self.stream.write(&cmd).unwrap(); // FIXME
    let mut buf = [0; 2048];
    self.stream.read(&mut buf).unwrap(); // FIXME
  }
}

