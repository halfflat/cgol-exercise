use std::error::Error;
use std::fmt;
use std::io;

//pub mod universe;
use crate::universe::Universe;


pub trait Renderer {
//    // Parse textual configuration data
//    fn parse_config(s: &str) -> Result<Box<dyn Any>, Box<dyn Error>>;
//
//    // Set-up / re-initialise
//    fn initialise(&mut self, extent: [u32; 2], config: Option<Box<dyn Any>>) -> Result<(), Box<dyn Error>>;

    // Can renders be printed to a terminal?
    fn tty(&self) -> bool { false }

    // If so, how many lines will it take up?
    fn tty_lines(&self) -> u32 { 0 }

    // Header
    fn write_prologue(&self, sink: &mut Box<dyn io::Write>) -> io::Result<()>;

    // Frame
    fn write_frame(&self, sink: &mut Box<dyn io::Write>, state: &Universe) -> io::Result<()>;

    // Tail
    fn write_epilogue(&self, sink: &mut Box<dyn io::Write>) -> io::Result<()>;
}

#[derive(Debug)]
pub enum RendererError {
    ConfigParseError(String, String),
//    ConfigTypeMismatch(String)
}

impl RendererError {
    pub fn config_parse_error(spec: impl Into<String>, reason: impl Into<String>) -> RendererError {
        RendererError::ConfigParseError(spec.into(), reason.into())
    }
//    fn config_type_mismatch(expected) -> RendererError {
//        RendererError::ConfigTypeMismatch(expected)
//    }
}

impl fmt::Display for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigParseError(spec, reason) => write!(f, "error parsing renderer configurations string '{spec}': {reason}") //,
//            Self::ConfigTypeMismatch(expected) => write!(f, "renderer configuration type mismatch, expected {expected}")
        }
    }
}

impl Error for RendererError {}

