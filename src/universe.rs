use std::fmt;
use std::mem;
use std::vec;

use crate::tile::{Tile, TileMut, Layout};


pub struct Universe {
    width: u32,
    height: u32,
    data: Vec<u8>,  // row-major ordered array, (width+2) × (height+2)
    next: Vec<u8>
}

impl Universe {
    // Public interface

    pub fn init(width: u32, height: u32) -> Self {
        let s = ((width+2)*(height+2)) as usize;
        Universe{width, height, data: vec![0u8; s], next: vec![0u8; s]}
    }

    pub fn size(&self) -> (u32, u32) { (self.width, self.height) }

    pub fn get_tile(&self, tile: &mut TileMut, at: [u32; 2]) {
        let full = Tile::tile(self.full_layout(), &self.data[..]);
        let inner = full.sub_tile([1, 1], [self.width, self.height]);
        let patch = inner.sub_tile(at, [tile.layout.columns, tile.layout.rows]);

        for j in 0..patch.layout.rows {
            for i in 0..patch.layout.columns {
                tile[[i,j]] = patch[[i,j]]
            }
        }

    }

    pub fn put_tile(&mut self, tile: &Tile, at: [u32; 2]) {
        let mut full = TileMut::tile(self.full_layout(), &mut self.data[..]);
        let mut inner = full.sub_tile_mut([1, 1], [self.width, self.height]);
        let mut patch = inner.sub_tile_mut(at, [tile.layout.columns, tile.layout.rows]);

        for j in 0..patch.layout.rows {
            for i in 0..patch.layout.columns {
                patch[[i,j]] = tile[[i,j]]
            }
        }
    }

    // Module-internal interface

    fn full_layout(&self) -> Layout { Layout{columns: self.width+2, stride: self.width+2, rows: self.height+2} }

    pub fn advance(&mut self) {
        // can't use Universe::view() above because of ownership.

        let cur = Tile::tile(self.full_layout(), &self.data[..]);
        let mut next = TileMut::tile(self.full_layout(), &mut self.next[..]);

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

        // Grab tile which includes right-side and bottom-side zero padding so that we don't have
        // to special case odd width or height in loop below.

        let g = Tile::tile(self.full_layout(), &self.data[..]).sub_tile([1, 1], [w+1, h+1]);

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
