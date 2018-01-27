// Copyright (c) 2018 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// Etherdream.rs, a library for the EtherDream laser projector DAC.

extern crate etherdream;

use etherdream::dac::Dac;
use etherdream::point::SimplePoint;

struct Square {
  /// Buffer of points
  pub prototype: Vec<SimplePoint>,
}

fn make_line(x1: f32, y1: f32, x2: f32, y2: f32, samples: u16)
    -> Vec<SimplePoint> {
  let xdiff = x1 - x2;
  let ydiff = y1 - y2;
  let mut points = Vec::with_capacity(samples as usize);

  for i in 0 .. samples {
    let j = i as f32 / samples as f32;
    let x = x1 - (xdiff * j);
    let y = y1 - (ydiff * j);
    points.push(SimplePoint::xy_red(x as i16, y as i16, 255));
  }

  points
}

impl Square {
  fn new(side: u16, samples: u16) -> Square {
    let mut prototype = Vec::with_capacity(samples as usize);
    let subsample = samples / 4;
    let half = side as f32 / 2.0;

    prototype.extend(make_line(-half, half, half, half, subsample));
    prototype.extend(make_line(half, half, half, -half, subsample));
    prototype.extend(make_line(half, -half, -half, -half, subsample));
    prototype.extend(make_line(-half, -half, -half, half, subsample));

    Square {
      prototype,
    }
  }

  fn get_points(&self, num_points: u16, pos: u32) -> Vec<SimplePoint> {
    let num_points = num_points as usize;
    let mut points = Vec::with_capacity(num_points);
    let mut i = pos as usize % self.prototype.len();

    while points.len() < num_points {
      let point = self.prototype.get(i).unwrap().clone(); // FIXME LAZINESS
      points.push(point);

      i = (i + 1) % self.prototype.len();
    }

    points
  }
}

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

  let square = Square::new(10_000, 300);
  let mut pos: u32 = 0;

  let _r = dac.stream_simple_points(|num_points: u16| {
    let points = square.get_points(num_points, pos);
    pos = pos + (points.len() as u32);
    points
  });
}
