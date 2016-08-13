// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>

use std::net::IpAddr;
use std::net::TcpStream;
use std::io::Read;
use std::io::Write;

struct Dac {
  ip_address: IpAddr,

}

impl Dac {

  /// CTOR.
  fn new(ip_address: IpAddr) -> Dac {
    Dac {
      ip_address: ip_address,
    }
  }

  fn write(&self) {
    let mut stream = TcpStream::connect((self.ip_address, 7765u16)).unwrap(); // FIXME

    let mut buf = [0; 2048];
    stream.write(&mut buf);

    stream.read(&mut buf);
  }
}

