// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// Etherdream.rs, a library for the EtherDream laser projector DAC.

extern crate etherdream;

use etherdream::dac::Dac;

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
  dac.play_demo();
}

