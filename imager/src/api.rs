// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use crate::opt;
pub use crate::data::{Resolution, OutputSize};


pub fn opt(input_image: &Vec<u8>, resize: OutputSize) -> Result<Vec<u8>, String> {
    let source = opt::Source::new(input_image, resize)?;
    let (output, opt_meta) = source.run_search();
    Ok(output)
}
