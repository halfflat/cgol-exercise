use std::env;
use std::fmt;
use std::mem;
use std::vec;
use std::ops::Index;
use std::ops::IndexMut;

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

fn main() {
    let mut u = Universe::init(4, 4);
    let mut m = u.mut_view();

    m[[1,1]] = 1u8;
    m[[2,1]] = 1u8;
    m[[2,2]] = 1u8;
    m[[3,2]] = 1u8;
    m[[4,4]] = 1u8;
//    m[[5,5]] = 1u8;

    println!("{u}.");
    u.advance();
    println!("{u}")
}
