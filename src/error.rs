// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// Etherdream.rs, a library for the EtherDream laser projector DAC.

//! Library errors

use protocol::AckCode;
use protocol::CommandCode;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::io::Error as IoError;

/// Represents all of the errors in the Etherdream library.
#[derive(Debug)]
pub enum EtherdreamError {
  /// Invalid length for an Etherdream response.
  BadResponseLength {
    /// Description of the error.
    description: String,
  },
  /// Network error.
  IoError {
    /// Cause of the error.
    cause: IoError,
  },
  /// Received a NACK from the EtherDream in response to a command.
  ReceivedNack {
    /// Type of NACK received.
    code: AckCode,
    /// The command the NACK was in response to.
    command: CommandCode,
  },
  /// We received a response for the wrong command.
  WrongResponse, // TODO: Encode more information in error?
}

impl Error for EtherdreamError {
  fn description(&self) -> &str {
    match *self {
      EtherdreamError::BadResponseLength { .. } => "BadResponseLength",
      EtherdreamError::IoError { .. } => "IoError",
      EtherdreamError::ReceivedNack { .. } => "ReceivedNack",
      EtherdreamError::WrongResponse => "WrongResponse",
    }
  }
}

impl Display for EtherdreamError {
  fn fmt(&self, f: &mut Formatter) -> Result {
    write!(f, "{}", self.description())
  }
}

impl From<IoError> for EtherdreamError {
  fn from(error: IoError) -> Self {
    EtherdreamError::IoError { cause: error }
  }
}
