// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use crate::opt;
use crate::data::OutputSize;
pub use crate::data::Resolution;


pub fn opt(input_image: &Vec<u8>, resize: Option<Resolution>) -> Result<Vec<u8>, String> {
    let resize = resize
        .map(|x| OutputSize::Px(x))
        .unwrap_or(OutputSize::default());
    let source = opt::Source::new(input_image, resize)?;
    let (output, opt_meta) = source.run_search();
    Ok(output)
}
