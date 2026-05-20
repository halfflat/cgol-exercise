// Render to utf8-encoded unicode characters with one glyph per site

use std::cmp::max;
use std::default::Default;
use std::io;
use std::str::FromStr;
use std::string::String;

pub mod render;
pub mod tile;
pub mod universe;

use crate::render::{Renderer, RendererError};
use crate::tile::{TileMut, Layout};
use crate::universe::Universe;
use std::io::Write;

// Each String in RenderCharsConfig represents a single grapheme.

#[derive(Clone)]
pub struct RenderCharsConfig {
    framed: bool,
    frame_chars: [String; 8], // top-left, top, top-right, left, right, bottom-left, bottom, bottom-right
    cell_chars: [String; 2]   // unalive, alive
}

// Helper to simplify setting up frame and cell char lists, with the assumption that there is one
// codepoint per glyph.

pub fn as_string_array<const LENGTH: usize>(s: &str) -> [String; LENGTH] {
//    assert!(s.chars().count() == LENGTH);
    <[String; LENGTH]>::try_from(s.chars().map(|c| String::from(c)).collect::<Vec<_>>()).unwrap()
}

impl Default for RenderCharsConfig {
    fn default() -> RenderCharsConfig {
        RenderCharsConfig{framed: true, frame_chars: as_string_array::<8>("┌-┐||└-┘"), cell_chars: as_string_array::<2>("·●")}
    }
}

impl FromStr for RenderCharsConfig {
    type Err = RendererError;
    fn from_str(s: &str) -> Result<Self, RendererError> {
        let mut c: RenderCharsConfig = Default::default();
        let mut spec = s;

        while !spec.is_empty() {
            if spec.starts_with("frame:") {
                spec = &spec[6..];
                if spec.chars().count() < 8 {
                    return Err(RendererError::config_parse_error(s, "require 8 frame characters after 'frame:'"))
                }
                c.framed = true;
                c.frame_chars = as_string_array::<8>(&spec[..8]);
                spec = &spec[8..];
                if spec.starts_with(',') {
                    spec = &spec[1..];
                    continue;
                }
            }
            if spec.starts_with("frame:") {
                spec = &spec[6..];
                c.framed = true;
                if spec.starts_with(',') {
                    spec = &spec[1..];
                    continue;
                }
            }
            else if spec.starts_with("cells:") {
                spec = &spec[6..];
                if spec.chars().count() < 2 {
                    return Err(RendererError::config_parse_error(s, "require 2 cell characters after 'cells:'"))
                }
                c.cell_chars = as_string_array::<2>(&spec[..2]);
                spec = &spec[2..];
                if spec.starts_with(',') {
                    spec = &spec[1..];
                    continue;
                }
            }
            else {
                return Err(RendererError::config_parse_error(s, "unrecognized text in config string"))
            }

        }
        Ok(c)
    }
}

pub struct RenderChars {
    config: RenderCharsConfig,
    extent: [u32; 2],
    max_glyph_bytes: u32,
    frame_top: String,
    frame_bottom: String
}

impl RenderChars {
    pub fn new(config: RenderCharsConfig, extent: [u32; 2]) -> Self {
        let mut max_glyph_bytes = 1u32;

        for i in 0..2 { max_glyph_bytes = max(max_glyph_bytes, config.cell_chars[i].len() as u32) }

        if config.framed {
            for i in 0..8 { max_glyph_bytes = max(max_glyph_bytes, config.frame_chars[i].len() as u32) }

            let mut frame_top = String::with_capacity(((2+extent[0])*max_glyph_bytes) as usize);
            let mut frame_bottom = String::with_capacity(((2+extent[0])*max_glyph_bytes) as usize);

            frame_top.push_str(&config.frame_chars[0]);
            for _ in 0..extent[0] { frame_top.push_str(&config.frame_chars[1]); }
            frame_top.push_str(&config.frame_chars[2]);

            frame_bottom.push_str(&config.frame_chars[5]);
            for _ in 0..extent[0] { frame_bottom.push_str(&config.frame_chars[6]); }
            frame_bottom.push_str(&config.frame_chars[7]);

            RenderChars{config, extent, max_glyph_bytes, frame_top, frame_bottom}
        }
        else {
            RenderChars{config, extent, max_glyph_bytes, frame_top: String::new(), frame_bottom: String::new()}
        }
    }
}

impl Renderer for RenderChars {
    fn tty(&self) -> bool { true }
    fn tty_lines(&self) -> u32 { self.extent[1] + if self.config.framed { 2 } else { 0 } }

    fn write_prologue(&self, _sink: &mut Box<dyn io::Write>) -> io::Result<()> { Ok(()) }
    fn write_epilogue(&self, _sink: &mut Box<dyn io::Write>) -> io::Result<()> { Ok(()) }

    fn write_frame(&self, sink: &mut Box<dyn io::Write>, state: &Universe) -> io::Result<()> {
        let w = self.extent[0];
        let h = self.extent[1];
        let mut frame = vec![0u8; (w*h) as usize];
        let mut g = TileMut::tile(Layout{columns: w, stride: w, rows: h}, &mut frame);

        state.get_tile(&mut g, [0u32, 0u32]);
        let mut buffered = io::BufWriter::with_capacity(((w+2)*self.max_glyph_bytes) as usize, sink);

        if self.config.framed { writeln!(buffered, "{}", self.frame_top)? }
        for j in 0..h {
            if self.config.framed { write!(buffered, "{}", self.config.frame_chars[3])? }
            for i in 0..w {
                write!(buffered, "{}", self.config.cell_chars[g[[i, j]] as usize])?
            }
            if self.config.framed { write!(buffered, "{}", self.config.frame_chars[3])? }
            writeln!(buffered, "")?
        }
        if self.config.framed { writeln!(buffered, "{}", self.frame_bottom)? }
        Ok(())
    }
}
