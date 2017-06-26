// Copyright (c) 2017 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// Etherdream.rs, a library for the EtherDream laser projector DAC.

extern crate etherdream;

use etherdream::dac::Dac;
use etherdream::protocol::Point;
use etherdream::protocol::X_MAX;
use etherdream::protocol::Y_MAX;
use std::f64::consts::PI;
use std::f64;

/// Number of points along the spiral to sample.
static SPIRAL_POINTS : i32 = 1000;

/// Number of points to blank from spiral's edge to center.
static BLANKING_POINTS : i32 = 20;

/// Other parameters.
static SPIRAL_GROWTH : f64 = 14.0;
static MAX_RADIUS : f64 = 326.0;

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

  let _r = dac.play_function(|num_points: u16| {
    let mut points = Vec::new();
    for _i in 0 .. num_points {
      // Get the current point along the beam.
      let f = unsafe {
        pos = (pos + 1) % (BLANKING_POINTS + SPIRAL_POINTS);
        pos
      };

      if f < SPIRAL_POINTS {
        let (x, y) = get_spiral_point(f);
        points.push(Point::xy_binary(x, y, true));
      } else {
        let (x, y) = get_blanking_point(f - SPIRAL_POINTS);
        points.push(Point::xy_binary(x, y, false));
      }
    }

    points
  });
}

fn get_spiral_point(cursor: i32) -> (i16, i16) {
  let i = (cursor as f64) / SPIRAL_POINTS as f64 * 2.0 * PI * SPIRAL_GROWTH;
  // Spirals are of the form A * x * trig(x), where A is constant.
  let x = i * i.cos() * MAX_RADIUS;
  let y = i * i.sin() * MAX_RADIUS;
  (x as i16, y as i16)
}

fn get_blanking_point(cursor: i32) -> (i16, i16) {
  // Get the outermost spiral point.
  let (end_x, end_y) = get_spiral_point(SPIRAL_POINTS);
  let x = end_x as f64 - end_x as f64 * cursor as f64 / BLANKING_POINTS as f64;
  let y = end_y as f64 - end_y as f64 * cursor as f64 / BLANKING_POINTS as f64;
  (x as i16, y as i16)
}
