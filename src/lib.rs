// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// Etherdream.rs, a library for the EtherDream laser projector DAC.

//! Etherdream.rs is a library for controlling the open source EtherDream laser
//! projector DAC hardware. This enables you to send a stream of points over the
//! network to the projector. All you have to do is provide code to generate
//! interesting points.

// TODO: Enable #![warn(missing_docs)]
#![deny(unused_extern_crates)]
#![deny(unused_imports)]
#![deny(unused_qualifications)]

extern crate byteorder;
extern crate net2;
extern crate point as pointlib;

mod error;

pub mod dac;
pub mod network;
pub mod protocol;

pub mod point {
  pub use pointlib::PipelinePoint;
  pub use pointlib::SimplePoint;
  pub use protocol::Point;
}

pub use error::EtherdreamError;
