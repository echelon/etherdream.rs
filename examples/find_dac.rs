// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// Etherdream.rs, a library for the EtherDream laser projector DAC.

extern crate etherdream;

fn main() {
  println!("Searching for DAC...");
  match etherdream::network::find_first_dac() {
    Err(e) => println!("Could not find DAC because of error: {}", e),
    Ok(result) => {
      println!("Found DAC at IP: {}", result.ip_address);
      println!("Broadcast: {:?}", result.broadcast);
    }
  }
}

