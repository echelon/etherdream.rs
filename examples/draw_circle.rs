// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// Etherdream.rs, a library for the EtherDream laser projector DAC.

extern crate byteorder;
extern crate etherdream;

use byteorder::LittleEndian;
use byteorder::WriteBytesExt;
use etherdream::dac::Dac;
use etherdream::protocol::Point;
use std::f64::consts::PI;
use std::f64;

// TODO TEMP: This is just lazy.
static mut J : i32 = 0;
static DIV : i32 = 200;

fn main() {
  println!("Searching for DAC...");

  let ip_addr = match etherdream::network::find_first_dac() {
    Err(e) => {
      println!("Could not find DAC because of error: {}", e);
      std::process::exit(0);
    },
    Ok(result) => {
      println!("Found DAC at IP: {}", result.ip_address);
      println!("Broadcast: {:?}", result.broadcast);
      result.ip_address
    },
  };

  let mut dac = Dac::new(ip_addr);
  //dac.play_demo();

  dac.play_function(|num_points: u16| {
    let mut cmd : Vec<u8> = Vec::new();
    cmd.push(0x64); // 'data' command.
    // TODO/FIXME: This should be LittleEndian. Why does this work only
    // as BigEndian!?
    cmd.write_u16::<LittleEndian>(num_points).unwrap();

    for _i in 0 .. num_points {
      // TODO TEMP
      let f = unsafe {
        J = (J + 1) % DIV;
        J
      };

      //let m = i as f64 * 1.0;
      let j = (f as f64 / DIV as f64) * 2 as f64 * PI;
      let x = j.cos() * 100.0f64;
      let y = j.sin() * 100.0f64;
      let pt = Point::xy_rgb(x as i16, y as i16, 255, 255, 255);
      //let pt = Point::xy_rgb(0, 0, 255, 255, 255);
      cmd.extend(pt.serialize());
    }

    cmd
  });
}

