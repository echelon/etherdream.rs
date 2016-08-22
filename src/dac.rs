// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>

use std::f64;
use byteorder::BigEndian;
use byteorder::LittleEndian;
use byteorder::WriteBytesExt;
use protocol::Begin;
use protocol::COMMAND_BEGIN;
use protocol::COMMAND_DATA;
use protocol::COMMAND_PING;
use protocol::COMMAND_PREPARE;
use protocol::DacResponse;
use protocol::DacStatus;
use protocol::Point;
use protocol::RESPONSE_ACK;
use protocol::RESPONSE_BUFFER_FULL;
use protocol::RESPONSE_INVALID_CMD;
use protocol::RESPONSE_STOP;
use std::f64::consts::PI;
use std::io::Read;
use std::io::Write;
use std::io;
use std::net::IpAddr;
use std::net::TcpStream;

// TODO TEMP
static mut J : u16 = 0;

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
    println!("\n");
    //self.hello();
    let mut response = self.read_response().unwrap();
    println!("Response: {:?}", response);

    self.try_prepare(response);


    loop {
      let mut started = false;
      
      let num_points = 1799 - response.status.buffer_fullness;

      println!("Sending {} points", num_points);

      println!("\nSend data");
      response = self.write_data(num_points).unwrap();
      println!("Response: {:?}", response);

      if !response.is_ack() {
        println!("Failure!");
        return;
      }


      if !started {
        println!("\nSend begin");
        let mut response = self.begin().unwrap();
        println!("Response: {:?}", response);
        if !response.is_ack() {
          println!("Failure!");
          return;
        }

        started = true;
      }
    }
  }

  fn hello(&mut self) -> Result<DacResponse, io::Error> {
    println!("Write hello");
    let cmd = [ COMMAND_PING ];
    self.stream.write(&cmd).unwrap(); // FIXME

    println!("Read hello ack");
    //let mut buf = [0; 2048];
    //self.stream.read(&mut buf).unwrap(); // FIXME
    self.read_response()
  }

  fn prepare(&mut self) -> Result<DacResponse, io::Error> {
    println!("Write prepare");
    let cmd = [ COMMAND_PREPARE ];
    self.stream.write(&cmd).unwrap(); // FIXME

    println!("Read prepare ack");
    //let mut buf = [0; 2048];
    //self.stream.read(&mut buf).unwrap(); // FIXME
    self.read_response()
  }

  fn begin(&mut self) -> Result<DacResponse, io::Error> {
    println!("Write begin");
    let cmd = Begin { low_water_mark: 0, point_rate: 30_000 };
    self.stream.write(&cmd.serialize()).unwrap(); // FIXME

    println!("Read begin ack");
    //let mut buf = [0; 2048];
    //self.stream.read(&mut buf).unwrap(); // FIXME
    self.read_response()
  }

  fn try_prepare(&mut self, response: DacResponse) {
    if response.status.playback_flags != 0x0 && response.status.playback_flags != 0x1 {
      println!("\nBad playback flags, must PREPARE: {}", response.status.playback_flags);
      println!("\nSend prepare");
      let resp = self.prepare().unwrap();
      println!("Response: {:?}", response);
      if !resp.is_ack() {
        println!("Failure!");
        panic!("Non-ACK received");
      }
      return;
    }

    if response.status.playback_state == 0x2 {
      println!("\nBad playback_state, must PREPARE: {}", response.status.playback_state);
      println!("\nSend prepare");
      let resp = self.prepare().unwrap();
      println!("Response: {:?}", response);
      if !resp.is_ack() {
        println!("Failure!");
        panic!("Non-ACK received");
      }
    }
  }

  /// Sends (3 + 18*n) bytes.
  fn write_data(&mut self, num_points: u16) -> Result<DacResponse, io::Error> {
    //let num_points = 10; // TODO

    let mut cmd : Vec<u8> = Vec::new();
    cmd.push(COMMAND_DATA);
    // TODO/FIXME: This should be LittleEndian. Why does this work only
    // as BigEndian!?
    cmd.write_u16::<LittleEndian>(num_points).unwrap();

    for i in 0 .. num_points {
      // TODO TEMP
      let f = unsafe {
        J = (J + 1) % 1_000;
        J
      };

      //let m = i as f64 * 1.0;
      let j = ((f as f64 * 1.0f64) / num_points as f64) * 2 as f64 * PI;
      let x = j.cos() * 100.0f64;
      let y = j.sin() * 100.0f64;
      let pt = Point::xy_rgb(x as i16, y as i16, 255, 255, 255);
      //let pt = Point::xy_rgb(0, 0, 255, 255, 255);
      cmd.extend(pt.serialize());
    }

    println!("Len: {}", cmd.len());
    println!("Write data");
    self.stream.write(&cmd).unwrap(); // FIXME

    println!("Read data ack");
    //let mut buf = [0; 2048];
    //self.stream.read(&mut buf).unwrap(); // FIXME
    self.read_response()
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

    let response = DacResponse::parse(&buf);

    match response {
      Ok(r) => println!("DacResponse: {:?}", r),
      Err(e) => println!("Error: {:?}", e),
    };
  }

  fn read_response(&mut self) -> Result<DacResponse, io::Error> {
    println!("read_response() ... ");
    let mut buf = [0; 22];

    match self.stream.read(&mut buf) {
      Ok(size) => {},
      Err(e) => {
        return Err(e);
      },
    }

    DacResponse::parse(&buf)
  }
}

