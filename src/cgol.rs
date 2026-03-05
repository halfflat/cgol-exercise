use std::error::Error;
use std::env;
use std::fmt;
use std::mem;
use std::process::ExitCode;
use std::thread;
use std::str::FromStr;
use std::time;
use std::vec;
use std::ops::Index;
use std::ops::IndexMut;

/*
// cgol width height [[--at X,Y] SOFCODE]...

enum SofElem {
    D(u32), // consecutive dead cells
    A(u32), // consecutive live cells
    L(u32)  // consecutive new lines
}

struct PatternSpec {
    x_offset: i32,
    y_offset: i32,
    elements: Vec<SofElem>
}

struct Options {
    width: u32,
    height: u32,
    patterns: Vec<PatternSpec>
}
*/

struct Universe {
    width: u32,
    height: u32,
    data: Vec<u8>,  // row-major ordered array, (width+2) × (height+2)
    next: Vec<u8>
}

struct UniverseView<'a> {
    stride: u32,
    dataref: &'a Vec<u8>
}

struct UniverseMutView<'a> {
    stride: u32,
    dataref: &'a mut Vec<u8>
}

impl<'a> Index<[u32; 2]> for UniverseView<'a> {
    type Output = u8;

    fn index(&self, ij: [u32; 2]) -> &u8 {
        let [i, j] = ij;
        &self.dataref[(i+self.stride*j) as usize]
    }
}

impl<'a> Index<[u32; 2]> for UniverseMutView<'a> {
    type Output = u8;

    fn index(&self, ij: [u32; 2]) -> &u8 {
        let [i, j] = ij;
        &self.dataref[(i+self.stride*j) as usize]
    }
}

impl<'a> IndexMut<[u32; 2]> for UniverseMutView<'a> {
    fn index_mut(&mut self, ij: [u32; 2]) -> &mut u8 {
        let [i, j] = ij;
        &mut self.dataref[(i+self.stride*j) as usize]
    }
}

impl Universe {
    fn init(width: u32, height: u32) -> Self {
        let s = ((width+2)*(height+2)) as usize;
        Universe{width, height, data: vec![0u8; s], next: vec![0u8; s]}
    }
    fn stride(&self) -> u32 { self.width+2 }
    fn view<'a>(&'a self) -> UniverseView<'a> { UniverseView{stride: self.stride(), dataref: &self.data} }
    fn mut_view<'a>(&'a mut self) -> UniverseMutView<'a> { UniverseMutView{stride: self.stride(), dataref: &mut self.data} }

    fn advance(&mut self) {
        // can't use Universe::view() above because of ownership.

        let cur = UniverseView{stride: self.stride(), dataref: &self.data};
        let mut next = UniverseMutView{stride: self.stride(), dataref: &mut self.next};

        for j in 1..=self.height {
            for i in 1..=self.width {
                let n: u8 = cur[[i-1, j-1]]+cur[[i,j-1]]+cur[[i+1,j-1]]+cur[[i-1,j]]+cur[[i+1,j]]+cur[[i-1,j+1]]+cur[[i,j+1]]+cur[[i+1,j+1]];
                let check: u8 = n*2 + cur[[i,j]];

                next[[i, j]] = match check { 5..=7 => 1, _ => 0 };
            }
        }
        mem::swap(&mut self.data, &mut self.next)
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pats: [char; 16] = [' ','▘','▝','▀','▖','▌','▞','▛','▗','▚','▐','▜','▄','▙','▟','█'];
        let w = self.width;
        let h = self.height;
        let g = self.view();

        writeln!(f, "┌{:─<1$}┐", "", ((w+1)/2) as usize)?;

        for j2 in 0..=(h-1)/2 {
            write!(f, "│")?;
            for i2 in 0..=(w-1)/2 {
                let i = 2*i2;
                let j = 2*j2;
                let k = g[[i,j]]+2*g[[i+1,j]]+4*g[[i,j+1]]+8*g[[i+1,j+1]];
                write!(f, "{}",pats[k as usize])?;
            }
            writeln!(f, "│")?
        }
        writeln!(f, "└{:─<1$}┘", "", ((w+1)/2) as usize)
    }
}

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
    // -t, --terminal
    opt_terminal: bool

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
                else if a == "-t" || a == "--terminal" {
                    opts.opt_terminal = true;
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
  -T, --terminal     use VT100 escape sequences in rendering

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
        let mut m = u.mut_view();

        m[[2,1]] = 1u8;
        m[[3,2]] = 1u8;
        m[[1,3]] = 1u8;
        m[[2,3]] = 1u8;
        m[[3,3]] = 1u8;

        let rc = if clopts.opt_terminal { "\x1b8" } else { "" };
        let sc = if clopts.opt_terminal { "\x1b7" } else { "" };

        if clopts.opt_terminal {
            let cuu = "\x1b[1A";
            // this feels so hacky
            let nlines = 3+(ny+1)/2;
            for _ in 0..nlines { print!("\n") }
            for _ in 0..nlines { print!("{cuu}") }
        }

        println!("{sc}{u}");
        for _ in 0..steps {
            if delay>0 { thread::sleep(time::Duration::from_millis(delay)) }
            u.advance();
            println!("{rc}{sc}{u}");
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
