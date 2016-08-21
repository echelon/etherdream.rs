// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>

use std::f64;
use std::f64::consts::PI;
use byteorder::LittleEndian;
use byteorder::BigEndian;
use byteorder::WriteBytesExt;
use protocol::Begin;
use protocol::COMMAND_BEGIN;
use protocol::COMMAND_DATA;
use protocol::COMMAND_PING;
use protocol::COMMAND_PREPARE;
use protocol::Point;
use protocol::RESPONSE_ACK;
use protocol::RESPONSE_BUFFER_FULL;
use protocol::RESPONSE_INVALID_CMD;
use protocol::RESPONSE_STOP;
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
    //self.hello();
    println!("Read hello");
    self.read();

    println!("Send begin");
    self.begin();

    println!("Send prepare");
    self.prepare();

    loop {
      println!("Send data");
      self.write_data();
    }
  }

  fn hello(&mut self) {
    println!("Write hello");
    let cmd = [ COMMAND_PING ];
    self.stream.write(&cmd).unwrap(); // FIXME

    println!("Read hello ack");
    //let mut buf = [0; 2048];
    //self.stream.read(&mut buf).unwrap(); // FIXME
    self.read();
  }

  fn prepare(&mut self) {
    println!("Write prepare");
    let cmd = [ COMMAND_PREPARE ];
    self.stream.write(&cmd).unwrap(); // FIXME

    println!("Read prepare ack");
    //let mut buf = [0; 2048];
    //self.stream.read(&mut buf).unwrap(); // FIXME
    self.read();
  }

  fn begin(&mut self) {
    println!("Write begin");
    let cmd = Begin { low_water_mark: 0, point_rate: 28_500 };
    self.stream.write(&cmd.serialize()).unwrap(); // FIXME

    println!("Read begin ack");
    //let mut buf = [0; 2048];
    //self.stream.read(&mut buf).unwrap(); // FIXME
    self.read();
  }

  fn write_data(&mut self) {
    let num_points = 100; // TODO

    let mut cmd : Vec<u8> = Vec::new();
    cmd.push(COMMAND_DATA);
    // TODO/FIXME: This should be LittleEndian. Why does this work only
    // as BigEndian!?
    cmd.write_i16::<BigEndian>(num_points).unwrap();

    // TODO WRITE POINTS
    for i in 0 .. num_points {
      //let m = i as f64 * 1.0;
      let j = ((i as f64 * 1.0f64) / num_points as f64) * 2 as f64 * PI;
      let x = j.cos() * 100.0f64;
      let y = j.sin() * 100.0f64;
      let pt = Point::xy_rgb(x as i16, y as i16, 255, 255, 255);
      cmd.extend(pt.serialize());
    }

    println!("Write data");
    self.stream.write(&cmd).unwrap(); // FIXME

    println!("Read data ack");
    //let mut buf = [0; 2048];
    //self.stream.read(&mut buf).unwrap(); // FIXME
    self.read();
  }

  fn read(&mut self) {
    println!("read() ... ");
    //let mut buf = Vec::new();
    let mut buf = [0; 22];
    match self.stream.read(&mut buf) {
      Ok(size) => {
        println!("Read bytes: {}", size);
      },
      Err(_) => {
        println!("Read error!");
        return;
      }
    }

    match buf[0] {
      RESPONSE_ACK => {
        println!("Read: ACK");
      },
      RESPONSE_BUFFER_FULL => {
        println!("Read: BUFFER_FULL");
      },
      RESPONSE_INVALID_CMD => {
        println!("Read: INVALID_CMD");
      },
      RESPONSE_STOP => {
        println!("Read: STOP");
      },
      _ => {
        println!("Unknown");
      },
    };

    match buf[1] {
      COMMAND_PING => {
        println!("Read: Ping");
      },
      COMMAND_BEGIN => {
        println!("Read: Begin");
      },
      COMMAND_DATA => {
        println!("Read: Data");
      },
      COMMAND_PREPARE => {
        println!("Read: Prepare");
      },
      _ => {
        println!("Read: Unknown");
      },
    }
  }
}

