// Chariot: An open source reimplementation of Age of Empires (1997)
// Copyright (c) 2016 Kevin Fuller
// Copyright (c) 2017 Taryn Hill
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! Start with [SlpFile](slp/struct.SlpFile.html) if you are decoding SLPs.
//!
//! ```ascii
//! +-----------------------------+
//! |          SlpHeader          |
//! +-----------------------------+
//! |SlpShapeHeader|SlpShapeHeader|
//! +-----------------------------+
//! |                             |
//! | Array of u16 padding pairs  | <-+ Each SlpShapeHeader has a "shape_outline_offset"
//! |                             |     that points to a pair in this array
//! +-----------------------------+
//! |                             |
//! | Arrays of u32 offsets to    | <-+ Each SlpShapeHeader has a "shape_data_offsets"
//! |  first command in each row  |     that points to an array
//! |                             |
//! +-----------------------------+
//! |                             |
//! | Drawing commands used to    |
//! |  produce indexed image data |
//! |                             |
//! +-----------------------------+
//! ```

#![recursion_limit = "1024"] // for the error_chain crate

#[macro_use]
extern crate error_chain;

extern crate chariot_io_tools;

mod error;
mod slp;

pub use error::ChainErr;
pub use error::Error;
pub use error::ErrorKind;
pub use error::Result;
pub use slp::SlpFile;
pub use slp::SlpHeader;
pub use slp::SlpLogicalShape;
pub use slp::SlpPixels;
pub use slp::SlpShapeHeader;
