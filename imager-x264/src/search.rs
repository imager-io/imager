// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::convert::AsRef;
use std::path::{PathBuf, Path};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::collections::VecDeque;
use libc::{size_t, c_float, c_void, fread};
use x264_dev::{raw, sys};
use itertools::Itertools;
use crate::yuv420p::Yuv420P;
use crate::stream::{Stream, FileStream, SingleImage};