// Copyright (c) 2017 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// Etherdream.rs, a library for the EtherDream laser projector DAC.

extern crate camera_capture;
extern crate etherdream;
extern crate image;

use etherdream::dac::Dac;
use etherdream::protocol::Point;
use etherdream::protocol::X_MAX;
use etherdream::protocol::X_MIN;
use etherdream::protocol::Y_MAX;
use etherdream::protocol::Y_MIN;
use etherdream::protocol::COLOR_MAX;
use camera_capture::Frame;
use std::f64::consts::PI;
use std::f64;
use image::ImageBuffer;
use image::Rgb;
use std::sync::Arc;
use std::sync::RwLock;

/// Number of points along the spiral to sample.
static SPIRAL_POINTS : i32 = 1500;

/// Number of points to blank from spiral's edge to center.
static BLANKING_POINTS : i32 = 20;

/// Other parameters.
static SPIRAL_GROWTH : f64 = 4.0;
static MIN_RADIUS: f64 = 500.0;
static MAX_RADIUS : f64 = 30000.0;

type Image = ImageBuffer<Rgb<u8>, Frame>;

fn main() {
  let frame_buffer : Arc<RwLock<Option<Image>>> = Arc::new(RwLock::new(None));
  let fb = frame_buffer.clone();

  //let (sender, receiver) = std::sync::mpsc::channel();

  let webcam_thread = std::thread::spawn(move || {
    let cam = camera_capture::create(0).expect("Could not open webcam.")
        .fps(30.0)
        .expect("Unsupported webcam fps.")
        .resolution(320, 240) // TODO param
        .expect("Unsupported webcam resolution.")
        .start()
        .expect("Could not begin webcam.");
    for frame in cam {
      /*if let Err(_) = sender.send(frame) {
        break;
      }*/
      match fb.write() {
        Err(_) => {},
        Ok(mut w) => {
          *w = Some(frame);
        }
      }
    }
  });

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
    //let frame = receiver.try_recv();
    let mut points = Vec::new();

    for _i in 0 .. num_points {
      // TODO: Let's build this into etherdream.rs
      // Get the current point along the beam.
      // TODO: Also, let's create a `dac.play_stream(S: Stream)`.
      let f = unsafe {
        pos = (pos + 1) % (BLANKING_POINTS + SPIRAL_POINTS);
        pos
      };

      match frame_buffer.read() {
        Err(_) => {},
        Ok(r) => {
          match *r {
            None => {
              if f < SPIRAL_POINTS {
                let (x, y) = get_spiral_point(f);
                points.push(Point::xy_binary(x, y, false));
              } else {
                let (x, y) = get_blanking_point(f - SPIRAL_POINTS);
                points.push(Point::xy_binary(x, y, false));
              }
            },
            Some(ref frame) => {

              if f < SPIRAL_POINTS {
                let (x, y) = get_spiral_point(f);
                let (r, g, b) = laser_color_from_webcam(&frame, x, y);
                points.push(Point::xy_rgb(x, y, r, g, b));
              } else {
                let (x, y) = get_blanking_point(f - SPIRAL_POINTS);
                points.push(Point::xy_binary(x, y, false));
              }

            },
          }
        },
      }


    }

    points
  });

  //drop(receiver);
  webcam_thread.join().unwrap();
}

fn get_spiral_point(cursor: i32) -> (i16, i16) {
  // TODO: FIX THIS MATH.
  let i = (cursor as f64) / SPIRAL_POINTS as f64;
  let t = i * 2.0 * PI * SPIRAL_GROWTH * 10.0;
  let r = MAX_RADIUS as f64 * i;
  // Spirals are of the form A * x * trig(x), where A is constant.
  let x = t.cos() * r;
  let y = t.sin() * r;
  (x as i16, y as i16)
}

fn get_blanking_point(cursor: i32) -> (i16, i16) {
  // Get the outermost spiral point.
  let (end_x, end_y) = get_spiral_point(SPIRAL_POINTS);
  let x = end_x as f64 - end_x as f64 * cursor as f64 / BLANKING_POINTS as f64;
  let y = end_y as f64 - end_y as f64 * cursor as f64 / BLANKING_POINTS as f64;
  (x as i16, y as i16)
}

fn laser_color_from_webcam(image: &Image, 
                           x: i16, y: i16) -> (u16, u16, u16) {

  fn webcam_x(laser_x: i16, image_width: u32) -> u32 {
    let num = laser_x as f64 - X_MIN as f64;
    let denom = X_MAX as f64 - X_MIN as f64;
    let ratio = num / denom;
    //println!("Ratio: {}", ratio);
    let scale = image_width as f64;
    let result = ratio * scale;
    result as u32
  }

  fn webcam_y(laser_y: i16, image_height : u32) -> u32 {
    let laser_y = laser_y * -1; // Inverted
    let num = laser_y as f64 - Y_MIN as f64;
    let denom = X_MAX as f64 - Y_MIN as f64;
    let ratio = num / denom;
    //println!("Ratio: {}", ratio);
    let scale = image_height as f64;
    let result = ratio * scale;
    result as u32
  }

  fn webcam_coord(laser_coord: i16) -> u32 {
    laser_coord as u32 // TODO
  }

  fn expand(color: u8) -> u16 {
    //(color as u16) << 8
    //(color as u16) * 257 // or the incorrect: (color as u16) << 8
    // Dithering
    if color < 100 {
      0
    } else {
      (color as u16) * 257 // or the incorrect: (color as u16) << 8
    }
  }    

  // -32768, 32767
  // to 
  // 0, 4294967295
  // but really
  // 0, img.width()
  //
  //let x_range : f64 = (x as i32 + 32768) as f64 / 65536.0;
  //let w_x = (x_range * image.width() as f64) as u32;

  //let y_range : f64 = (x as i32 + 32768) as f64 / 65536.0;
  //let w_y = (y_range * image.height() as f64) as u32;
  
  //println!("x, y: {}, {}", x, y);
  let w_x = webcam_x(x, image.width());
  let w_y = webcam_y(y, image.height());

  //println!("w_x, w_y: {}, {}", w_x, w_y);

  let pix = image.get_pixel(w_x, w_y);

  //println!("Pixel: {:?}", pix);

  let r = expand(pix.data[0]);
  let g = expand(pix.data[1]);
  let b = expand(pix.data[2]);


  /*if x % 10 == 0 {
    println!("xy: {}, {} | rgb: {}, {}, {}", w_x, w_y, r, g, b);
  }*/

  (r, g, b)
}

