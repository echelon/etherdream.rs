// Copyright (c) 2017 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// Etherdream.rs, a library for the EtherDream laser projector DAC.

extern crate etherdream;

use etherdream::dac::Dac;
use etherdream::point::PipelinePoint;
use etherdream::point::Point;
use etherdream::protocol::X_MAX;
use etherdream::protocol::Y_MAX;
use std::f64::consts::PI;
use std::f64;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

static DIV : i32 = 200;

struct Square {
  /// Square x coordinate
  pub x_coordinate: f32,
  /// Square y coordinate
  pub y_coordinate: f32,
  /// Size of the square
  pub side: f32,
  /// Spin angle
  pub theta: f32,
  /// Buffer of points
  pub prototype: Vec<PipelinePoint>,
}

fn make_line(x1: f32, y1: f32, x2: f32, y2: f32, samples: u16)
    -> Vec<PipelinePoint> {
  let xdiff = x1 - x2;
  let ydiff = y1 - y2;
  let mut points = Vec::with_capacity(samples as usize);

  for i in 0 .. samples {
    let j = i as f32 / samples as f32;
    let x = x1 - (xdiff * j);
    let y = y1 - (ydiff * j);
    points.push(PipelinePoint::xy_binary(x, y, true));
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
      x_coordinate: 0.0,
      y_coordinate: 0.0,
      side: 5_000.0,
      theta: 0.0,
      prototype,
    }
  }

  fn get_points(&self, num_points: u16, pos: i32) -> Vec<PipelinePoint> {
    let mut points = Vec::with_capacity(num_points as usize);
    let num_points = num_points as usize;
    //let mut i = pos as usize % self.prototype.len();

    while points.len() < num_points {
      //let point = self.prototype.get(i).unwrap().clone(); // FIXME LAZINESS
      //points.push(point);
      points.push(PipelinePoint::xy_binary(10_000.0, 10_000.0, true));

      //i = (i + 1) % self.prototype.len();
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

  let mut square = Arc::new(RwLock::new(Square::new(5000, 300)));
  let mut square2 = square.clone();

  // le animation thread.
  thread::spawn(move || {
    loop {
      match square.write() {
        Err(_) => {},
        Ok(square) => {
          // TODO: Move square.
        }
      }

      thread::sleep(Duration::from_millis(100));
    }
  });

  let mut pos: i32 = 0;

  let _r = dac.stream_pipeline_points(|num_points: u16| {
    let mut points = Vec::with_capacity(num_points as usize);

    match square2.read() {
      Err(_) => {},
      Ok(ref square) => {
        points = square.get_points(num_points, pos);
      }
    }

    points
  });
}
