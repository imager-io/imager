// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::str::FromStr;
use serde::{Serialize, Deserialize};

///////////////////////////////////////////////////////////////////////////////
// OUTPUT-FORMAT
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputFormat {
    Jpeg,
}

impl FromStr for OutputFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "jpeg" => Ok(OutputFormat::Jpeg),
            "jpg" => Ok(OutputFormat::Jpeg),
            _ => {
                Err(format!("Unknown or unsupported output format {}", s))
            }
        }
    }
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Jpeg
    }
}

///////////////////////////////////////////////////////////////////////////////
// RESOLUTION
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub fn new(width: u32, height: u32) -> Self {
        Resolution{width, height}
    }
}

impl std::fmt::Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}


impl FromStr for Resolution {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let ix = input.find("x").ok_or("invalid")?;
        let (width, height) = input.split_at(ix);
        let height = height.trim_start_matches("x");
        let width = u32::from_str(width).map_err(|_| "invalid")?;
        let height = u32::from_str(height).map_err(|_| "invalid")?;
        Ok(Resolution {width, height})
    }
}



///////////////////////////////////////////////////////////////////////////////
// OUTPUT-SIZE
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
pub enum OutputSize {
    /// Output image resolution. Akin to the 'px' CSS unit.
    Px(Resolution),
    /// Retain the original resolution. Akin to the '100%' CSS value.
    Full,
}

impl std::fmt::Display for OutputSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputSize::Px(px) => write!(f, "{}", px),
            OutputSize::Full => write!(f, "full"),
        }
    }
}

impl FromStr for OutputSize {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "full" => Ok(OutputSize::Full),
            _ => {
                let val: Resolution = Resolution::from_str(s)?;
                Ok(OutputSize::Px(val))
            }
        }
    }
}

impl Default for OutputSize {
    fn default() -> Self {
        OutputSize::Full
    }
}


impl Serialize for OutputSize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for OutputSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        String::deserialize(deserializer)?.parse().map_err(serde::de::Error::custom)
    }
}
