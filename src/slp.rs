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
//

use error::*;

use chariot_io_tools::ReadExt;
use std::fs::File;
use std::io::SeekFrom;
use std::io::prelude::*;
use std::mem::size_of;

use std::path::Path;

/// A struct containing SLP metadata.
///
/// A single SlpHeader must exist at the beginning of an [SlpFile](struct.SlpFile.html).
pub struct SlpHeader {
    /// This should always be `2.0N`
    pub file_version: [u8; 4],
    pub shape_count: u32,
    pub comment: [u8; 24],
}

impl SlpHeader {
    pub fn new() -> SlpHeader {
        SlpHeader {
            file_version: [0u8; 4],
            shape_count: 0u32,
            comment: [0u8; 24],
        }
    }

    // TODO: Implement writing

    pub fn read_from<S: Read>(stream: &mut S) -> Result<SlpHeader> {
        let mut header = SlpHeader::new();
        try!(stream.read_exact(&mut header.file_version));
        header.shape_count = try!(stream.read_u32());
        try!(stream.read_exact(&mut header.comment));

        if header.file_version != "2.0N".as_bytes() {
            return Err(ErrorKind::InvalidSlp("bad header".into()).into());
        }
        Ok(header)
    }
}

#[derive(Debug)]
/// A 32-byte struct containing frame metadata.
///
/// One of these will exists for every frame in an [SlpFile](struct.SlpFile.html).
pub struct SlpShapeHeader {
    /// Pointer to an array of offsets.
    ///
    /// Each offset defines the position of the first command of a row.
    ///
    /// The first offset in this array is the first drawing command for the image.
    pub shape_data_offsets: u32,

    /// Pointer to an array of u16 pairs used to indicate padding.
    pub shape_outline_offset: u32,
    pub palette_offset: u32,
    pub properties: u32,
    pub width: u32,
    pub height: u32,
    pub center_x: i32,
    pub center_y: i32,
}

impl SlpShapeHeader {
    pub fn new() -> SlpShapeHeader {
        SlpShapeHeader {
            shape_data_offsets: 0u32,
            shape_outline_offset: 0u32,
            palette_offset: 0u32,
            properties: 0u32,
            width: 0u32,
            height: 0u32,
            center_x: 0i32,
            center_y: 0i32,
        }
    }

    // TODO: Implement writing

    fn read_from_file<R: Read + Seek>(file: &mut R) -> Result<SlpShapeHeader> {
        let mut header = SlpShapeHeader::new();
        header.shape_data_offsets = try!(file.read_u32());
        header.shape_outline_offset = try!(file.read_u32());
        header.palette_offset = try!(file.read_u32());
        header.properties = try!(file.read_u32());
        header.width = try!(file.read_u32());
        header.height = try!(file.read_u32());
        header.center_x = try!(file.read_i32());
        header.center_y = try!(file.read_i32());
        Ok(header)
    }
}

pub type SlpPixels = Vec<u8>;
pub type SlpDrawCommands = Vec<DrawCommand>;

#[derive(Clone)]
pub enum DrawCommand {
    Color,
    Skip,
    Remap,
    Shadow,
}

pub struct SlpLogicalShape {
    pub header: SlpShapeHeader,
    pub pixels: SlpPixels,
    pub commands: SlpDrawCommands,
}

impl SlpLogicalShape {
    pub fn new() -> SlpLogicalShape {
        SlpLogicalShape {
            header: SlpShapeHeader::new(),
            pixels: SlpPixels::new(),
            commands: SlpDrawCommands::new(),
        }
    }
}

enum SlpEncodedLength {
    SixUpperBit,
    FourUpperBit,
    LargeLength,
}

impl SlpEncodedLength {
    fn decode<R: Read>(self, cmd_byte: u8, cursor: &mut R) -> Result<usize> {
        match self {
            SlpEncodedLength::SixUpperBit => {
                let length = (cmd_byte >> 2) as usize;
                if length == 0 {
                    return Err(ErrorKind::BadLength.into());
                }
                Ok(length)
            }
            SlpEncodedLength::FourUpperBit => {
                let mut length = (cmd_byte >> 4) as usize;
                if length == 0 {
                    length = try!(cursor.read_u8()) as usize;
                }
                Ok(length)
            }
            SlpEncodedLength::LargeLength => {
                let mut length = ((cmd_byte & 0xF0) as usize) << 4;
                length += try!(cursor.read_u8()) as usize;
                Ok(length)
            }
        }
    }
}

/// An image container format written by Ensemble Studios for their "Genie" game engine.
///
/// An SLP is made up of a header and numerous frames (sometimes called "shapes").
pub struct SlpFile {
    pub header: SlpHeader,
    pub shapes: Vec<SlpLogicalShape>,

    // TODO: Remove this from SlpFile.
    // We shouldn't be comitting to a player index until we hit the fragment shader.
    pub player_index: u8,
}

impl SlpFile {
    pub fn new(player_index: u8) -> SlpFile {
        SlpFile {
            header: SlpHeader::new(),
            shapes: Vec::new(),
            player_index: player_index,
        }
    }

    // TODO: Implement writing

    pub fn read_from_file<P: AsRef<Path>>(file_name: P, player_index: u8) -> Result<SlpFile> {
        let file_name = file_name.as_ref();
        let mut file = try!(File::open(file_name));
        return SlpFile::read_from(&mut file, player_index);
    }

    pub fn read_from<R: Read + Seek>(cursor: &mut R, player_index: u8) -> Result<SlpFile> {
        let mut slp_file = SlpFile::new(player_index);
        slp_file.header = try!(SlpHeader::read_from(cursor));
        for _shape_index in 0..slp_file.header.shape_count {
            let mut shape = SlpLogicalShape::new();
            shape.header = try!(SlpShapeHeader::read_from_file(cursor));
            slp_file.shapes.push(shape);
        }

        for shape in &mut slp_file.shapes {
            try!(SlpFile::read_pixel_data(cursor, shape, player_index));
        }

        Ok(slp_file)
    }

    fn read_pixel_data<R: Read + Seek>(cursor: &mut R,
                                       shape: &mut SlpLogicalShape,
                                       player_index: u8)
                                       -> Result<()> {
        let width = shape.header.width;
        let height = shape.header.height;

        // Reserve and zero out pixel data
        shape.pixels.resize((width * height) as usize, 0u8);
        shape.commands.resize((width * height) as usize, DrawCommand::Skip);

        for y in 0..height {
            let line_outline_offset = shape.header.shape_outline_offset + (y * size_of::<u32>() as u32);

            try!(cursor.seek(SeekFrom::Start(line_outline_offset as u64)));
            let mut x = try!(cursor.read_u16()) as u32;
            let right_padding = try!(cursor.read_u16()) as u32;
            if x == 0x8000 || right_padding == 0x8000 {
                // Fully transparent; skip to next line
                continue;
            }

            // The shape_data_offset points to an array of offsets to actual pixel data
            // Seek out the offset for the current Y coordinate
            let shape_data_ptr_offset = shape.header.shape_data_offsets + (y * size_of::<u32>() as u32);
            try!(cursor.seek(SeekFrom::Start(shape_data_ptr_offset as u64)));

            // Read the offset and seek to it so we can see the actual data
            let data_offset = try!(cursor.read_u32());
            try!(cursor.seek(SeekFrom::Start(data_offset as u64)));

            // TODO: Consider detecting endless loop when we loop more times than there are pixels
            loop {
                let cmd_byte = try!(cursor.read_u8());

                // End of line indicator
                if cmd_byte == 0x0F {
                    if x != width - right_padding {
                        return Err(ErrorKind::InvalidSlp(format!("Line {} not the expected \
                                                                  size. Was {} but should be {}",
                                                                 y,
                                                                 x,
                                                                 width - right_padding))
                            .into());
                    }
                    break;
                }

                if x > width {
                    return Err(ErrorKind::InvalidSlp("Unexpected error occurred.
                        Line length already exceeded before stop."
                            .into())
                        .into());
                }

                use self::SlpEncodedLength::*;

                // SLP opcodes are stored in the lowest 4 bits of the command byte.
                let opcode = cmd_byte & 0b00001111;

                match opcode {
                    // Block copy
                    0b0000 |
                    0b0100 |
                    0b1000 |
                    0b1100 => {
                        let length = try!(SixUpperBit.decode(cmd_byte, cursor));
                        for _ in 0..length {
                            let loc = (y * width + x) as usize;
                            shape.pixels[loc] = try!(cursor.read_u8());
                            shape.commands[loc] = DrawCommand::Color;
                            x += 1;
                        }
                    }

                    // Skip pixels
                    0b0001 |
                    0b0101 |
                    0b1001 |
                    0b1101 => {
                        x += try!(SixUpperBit.decode(cmd_byte, cursor)) as u32;
                    }

                    // Large block copy
                    0b0010 => {
                        let length = try!(LargeLength.decode(cmd_byte, cursor));
                        for _ in 0..length {
                            let loc = (y * width + x) as usize;
                            shape.pixels[loc] = try!(cursor.read_u8());
                            shape.commands[loc] = DrawCommand::Color;
                            x += 1;
                        }
                    }

                    // Large skip pixels
                    0b0011 => {
                        let length = try!(LargeLength.decode(cmd_byte, cursor));
                        x += length as u32;
                    }

                    // Copy and colorize block
                    0b0110 => {
                        let length = try!(FourUpperBit.decode(cmd_byte, cursor));

                        for _ in 0..length {
                            let relative_index = try!(cursor.read_u8());
                            let player_color = player_index * 16 + relative_index;
                            let loc = (y * width + x) as usize;
                            shape.pixels[loc] = player_color | relative_index;
                            shape.commands[loc] = DrawCommand::Remap;
                            x += 1;
                        }
                    }

                    // Fill block
                    0b0111 => {
                        let length = try!(FourUpperBit.decode(cmd_byte, cursor));
                        let color = try!(cursor.read_u8());
                        for _ in 0..length {
                            let loc = (y * width + x) as usize;
                            shape.pixels[loc] = color;
                            shape.commands[loc] = DrawCommand::Color;
                            x += 1;
                        }
                    }

                    // Transform block
                    0b1010 => {
                        let length = try!(FourUpperBit.decode(cmd_byte, cursor));
                        let relative_index = try!(cursor.read_u8());
                        let player_color = player_index * 16 + relative_index;

                        for _ in 0..length {
                            let loc = (y * width + x) as usize;
                            shape.pixels[loc] = player_color | relative_index;
                            shape.commands[loc] = DrawCommand::Remap;
                            x += 1;
                        }
                    }

                    // Shadow pixels
                    0b1011 => {
                        let length = try!(FourUpperBit.decode(cmd_byte, cursor));
                        for _ in 0..length {
                            let loc = (y * width + x) as usize;
                            shape.commands[loc] = DrawCommand::Shadow;
                        }
                    }

                    // Extended
                    0b1110 => {
                        // The extended opcode lives in the top 4 bits of the command byte (yes, I lied above).
                        let opcode = cmd_byte & 0b11110000;
                        panic!("Extended (0x0E) not implemented (cmd_byte={}, opcode={})", cmd_byte, opcode);
                    }

                    _ => panic!("unknown command: {}", cmd_byte),
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{SlpHeader, ErrorKind};

    #[test]
    fn test_slp_header_read_from() {
        use std::io;
        let data = "2.0N\x04\0\0\0test\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".as_bytes();
        let result = SlpHeader::read_from(&mut io::Cursor::new(data));
        match result {
            Ok(slp_header) => assert_eq!(4u32, slp_header.shape_count),
            Err(e) => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn test_slp_header_read_from_bad_header() {
        use std::io;
        let data = "2.1N\x04\0\0\0test\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".as_bytes();
        let result = SlpHeader::read_from(&mut io::Cursor::new(data));
        match result {
            Ok(_) => panic!("expected bad header error"),
            Err(e) => {
                match e.kind() {
                    &ErrorKind::InvalidSlp(ref reason) => assert_eq!(*reason, "bad header".to_string()),
                    _ => panic!("unexpected error: {}", e),
                }
            }
        }
    }
}
