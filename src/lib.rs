// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// Etherdream.rs, a library for the EtherDream laser projector DAC.

//! Etherdream.rs is a library for controlling the open source EtherDream laser
//! projector DAC hardware. This enables you to send a stream of points over the
//! network to the projector. All you have to do is provide code to generate
//! interesting points.

// TODO: Enable #![warn(missing_docs)]
#![warn(unused_extern_crates)]
#![warn(unused_qualifications)]

extern crate byteorder;
extern crate net2;

mod error;

pub mod dac;
pub mod network;
pub mod protocol;

pub use error::EtherdreamError;
