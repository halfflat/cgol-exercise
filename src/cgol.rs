use std::error::Error;
use std::env;
use std::fmt;
use std::io;
use std::process::ExitCode;
use std::thread;
use std::str::FromStr;
use std::time;

#[path = "render-chars.rs"]
pub mod render_chars;
pub mod render;
pub mod tile;
pub mod universe;

use crate::render_chars::{RenderChars,RenderCharsConfig};
use crate::render::Renderer;
use crate::tile::{Tile, Layout};
use crate::universe::Universe;
use std::io::Write;

fn parse_1or2<T: FromStr + Copy>(s: &str) -> Result<(T, T), <T as FromStr>::Err> {
    if let Some(comma) = s.find(',') {
        let left: T = s[..comma].parse()?;
        let right: T = s[comma+1..].parse()?;
        Ok((left, right))
    }
    else {
        let n: T = s.parse()?;
        Ok((n, n))
    }
}

#[derive(Default)]
#[derive(Debug)]
struct CLOptions {
    // -h, --help
    opt_help: bool,
    // -s, --size X[,Y]
    opt_size: Option<(u32,u32)>,
    // -N, --steps N
    opt_steps: Option<u32>,
    // -d, --delay MILLISECONDS
    opt_delay: Option<u32>,
    // -t, --tty
    opt_tty: bool

    // More options for the future...
    //
    // [--at X,Y] --rle RLEDATA
    // [--at X,Y] --sof SOFDATA
    // -r, --render ascii|block|none
    // -i, --input PBMFILE
    // -o, --output PBMFILE
    // -O, --output-all PBMFILE
}

#[derive(Debug)]
enum ParseCLOptionsError {
    ParseError(String, String), // argument, context
    Missing(String), // context
    Unexpected(String) // argument
}

impl ParseCLOptionsError {
    fn parse_error(arg: impl Into<String>, context: impl Into<String>) -> ParseCLOptionsError {
        ParseCLOptionsError::ParseError(arg.into(), context.into())
    }

    fn missing(context: impl Into<String>) -> ParseCLOptionsError {
        ParseCLOptionsError::Missing(context.into())
    }

    fn unexpected(arg: impl Into<String>) -> ParseCLOptionsError {
        ParseCLOptionsError::Unexpected(arg.into())
    }
}

impl fmt::Display for ParseCLOptionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError(arg, context) => write!(f, "error parsing argument for {context}: {arg}"),
            Self::Missing(context) => write!(f, "missing argument for {context}"),
            Self::Unexpected(arg) => write!(f, "unexpected argument: {arg}")
        }
    }
}

impl Error for ParseCLOptionsError {}

fn parse_clopts_from_args() -> Result<CLOptions, ParseCLOptionsError> {
    let mut opts: CLOptions = Default::default();

    let argv: Vec<String> = env::args().collect();
    let argc = argv.len();

    if argc<=1 { return Ok(opts) }

    enum State { NoOpt, OptSize, OptSteps, OptDelay }
    let mut state: State = State::NoOpt;

    let mut i = 1;
    let mut a = argv[i].as_str();
    let mut arg_context = "";

    loop {
        match state {
            State::NoOpt => {
                if a == "-s" || a == "--size" {
                    arg_context = a;
                    state = State::OptSize;
                }
                else if a.starts_with("--size=") {
                    let k = "--size".len();
                    arg_context = &a[..k];
                    a = &a[k+1..];
                    state = State::OptSize;
                    continue;
                }
                else if a == "-N" || a == "--steps" {
                    arg_context = a;
                    state = State::OptSteps;
                }
                else if a.starts_with("--steps=") {
                    let k = "--steps".len();
                    arg_context = &a[..k];
                    a = &a[k+1..];
                    state = State::OptSteps;
                    continue;
                }
                else if a == "-d" || a == "--delay" {
                    arg_context = a;
                    state = State::OptDelay;
                }
                else if a.starts_with("--delay=") {
                    let k = "--delay".len();
                    arg_context = &a[..k];
                    a = &a[k+1..];
                    state = State::OptDelay;
                    continue;
                }
                else if a == "-h" || a == "--help" {
                    opts.opt_help = true;
                    state = State::NoOpt
                }
                else if a == "-t" || a == "--tty" {
                    opts.opt_tty = true;
                    state = State::NoOpt
                }
                else {
                    return Err(ParseCLOptionsError::unexpected(a));
                }
            },
            State::OptSize => {
                opts.opt_size = Some(parse_1or2::<u32>(a).map_err(|e| ParseCLOptionsError::parse_error(e.to_string(), arg_context))?);
                state = State::NoOpt
            },
            State::OptSteps => {
                opts.opt_steps = Some(a.parse::<u32>().map_err(|e| ParseCLOptionsError::parse_error(e.to_string(), arg_context))?);
                state = State::NoOpt
            }
            State::OptDelay => {
                opts.opt_delay = Some(a.parse::<u32>().map_err(|e| ParseCLOptionsError::parse_error(e.to_string(), arg_context))?);
                state = State::NoOpt
            }
        }

        i = i+1;
        if i>=argc { break }

        a = argv[i].as_str();
    }

    match state {
        State::NoOpt => Ok(opts),
        _ => Err(ParseCLOptionsError::missing(arg_context))
    }
}

fn invoked_as() -> String {
    let a0 = env::args().next().unwrap();
    String::from(&a0.as_str()[a0.rfind('/').map(|i| i+1).unwrap_or(0)..])
}

static USAGE: &str = "[OPTION]...
  -s, --size=X[,Y]   specify domain size
  -N, --steps=N      run for N steps
  -d, --delay=TIME   delay TIME ms between steps
  -t, --tty          use VT100 escape sequences in rendering

  -h, --help         display this help and exit";

fn main() -> ExitCode {
    let inner = || -> Result<(), Box<dyn Error>> {
        let clopts = parse_clopts_from_args()?;

        if clopts.opt_help {
            println!("Usage: {} {}", invoked_as(), USAGE);
            return Ok(())
        }

        let (nx, ny) = clopts.opt_size.unwrap_or((41,41));
        let delay = clopts.opt_delay.unwrap_or(0) as u64;
        let steps = clopts.opt_steps.unwrap_or(std::u32::MAX);

        let mut u = Universe::init(nx, ny);

        let glider = vec![0u8, 1u8, 0u8, 0u8, 0u8, 1u8, 1u8, 1u8, 1u8];
        let glider_tile = Tile::tile(Layout{columns: 3, stride: 3, rows: 3}, &glider[..]);

        u.put_tile(&glider_tile, [1, 1]);

        let rc = if clopts.opt_tty { "\x1b8" } else { "" };
        let sc = if clopts.opt_tty { "\x1b7" } else { "" };

        let renderer = Box::new(RenderChars::new(Default::default(), [nx, ny]));

        if clopts.opt_tty {
            let mut out: Box<dyn Write> = Box::new(io::stdout());

            let cuu = "\x1b[1A";
            let nlines = renderer.tty_lines();
            for _ in 0..nlines { print!("\n") }
            for _ in 0..nlines { print!("{cuu}") }

            renderer.write_prologue(&mut out)?;
            print!("{sc}");
            renderer.write_frame(&mut out, &u)?;
            for _ in 0..steps {
                if delay>0 { thread::sleep(time::Duration::from_millis(delay)) }
                u.advance();
                print!("{rc}{sc}");
                renderer.write_frame(&mut out, &u)?;
            }
            renderer.write_epilogue(&mut out)?;
        }
        else {
            let mut out: Box<dyn Write> = Box::new(io::stdout());

            renderer.write_prologue(&mut out)?;
            renderer.write_frame(&mut out, &u)?;
            for _ in 0..steps {
                if delay>0 { thread::sleep(time::Duration::from_millis(delay)) }
                u.advance();
                renderer.write_frame(&mut out, &u)?;
            }
            renderer.write_epilogue(&mut out)?;
        }

        Ok(())
    };

    match inner() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) if e.is::<ParseCLOptionsError>() => {
            eprintln!("{}: {}\nTry --help for more information.", invoked_as(), e);
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("{}: {}", invoked_as(), e);
            ExitCode::FAILURE
        }
    }
}
