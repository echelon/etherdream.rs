// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// Etherdream.rs, a library for the EtherDream laser projector DAC.

extern crate etherdream;

use etherdream::dac::Dac;
use etherdream::protocol::Point;
use std::f64::consts::PI;
use std::f64;

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

  static mut pos: i32 = 0;

  dac.play_function(|num_points: u16| {
    let mut points = Vec::new();
    for _i in 0 .. num_points {
      let f = unsafe {
        pos = (pos + 1) % DIV;
        pos
      };

      let j = (f as f64 / DIV as f64) * 2 as f64 * PI;
      let x = j.cos() * 100.0f64;
      let y = j.sin() * 100.0f64;

      points.push(Point::xy_rgb(x as i16, y as i16, 255, 255, 255));
    }

    points
  });
}

