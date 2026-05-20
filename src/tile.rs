use std::cmp::min;
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub columns: u32,
    pub stride: u32,
    pub rows: u32
}

impl Layout {
    pub fn size(&self) -> usize {
        if self.rows == 0 { 0 }
        else {
            (self.stride as usize)*((self.rows-1) as usize) + self.columns as usize
        }
    }

    pub fn empty(&self) -> bool { self.rows == 0 || self.columns == 0 }

    fn offset(&self, at: [u32; 2]) -> Option<usize> {
        if at[0] >= self.columns || at[1] >= self.rows { None }
        else { Some((at[0] as usize)*(self.stride as usize) + at[1] as usize) }
    }

    pub fn sub_layout(&self, at: [u32; 2], extent: [u32; 2]) -> Layout {
        let max_nc: u32 = if at[0] >= self.columns { 0 } else { self.columns - at[0] };
        let max_nr: u32 = if at[1] >= self.rows { 0 } else { self.columns - at[1] };

        Layout{columns: min(extent[0], max_nc), stride: self.stride, rows: min(extent[1], max_nr)}
    }
}

pub struct Tile<'a> {
    pub layout: Layout,
    pub data: &'a [u8]
}

pub struct TileMut<'a> {
    pub layout: Layout,
    pub data: &'a mut [u8]
}

impl<'a> Tile<'a> {
    pub fn tile(layout: Layout, data: &'a [u8]) -> Tile<'a> { Tile{layout, data} }

    pub fn empty(&self) -> bool { self.layout.empty() }

    pub fn sub_tile(&self, at: [u32; 2], extent: [u32; 2]) -> Tile<'a> {
        let i = self.layout.offset(at).unwrap_or(0usize);
        let subl = self.layout.sub_layout(at, extent);

        Tile{layout: subl, data: &self.data[i..i+subl.size()]}
    }
}

impl<'a> TileMut<'a> {
    #[allow(dead_code)]

    pub fn tile(layout: Layout, data: &'a mut [u8]) -> TileMut<'a> { TileMut{layout, data} }

    pub fn empty(&self) -> bool { self.layout.empty() }

    pub fn sub_tile(&'a self, at: [u32; 2], extent: [u32; 2]) -> Tile<'a> {
        let i = self.layout.offset(at).unwrap_or(0usize);
        let subl = self.layout.sub_layout(at, extent);
        Tile{layout: subl, data: &self.data[i..i+subl.size()]}
    }

    pub fn sub_tile_mut(&'a mut self, at: [u32; 2], extent: [u32; 2]) -> TileMut<'a> {
        let i = self.layout.offset(at).unwrap_or(0usize);
        let subl = self.layout.sub_layout(at, extent);
        TileMut{layout: subl, data: &mut self.data[i..i+subl.size()]}
    }
}


impl<'a> Index<[u32; 2]> for Tile<'a> {
    type Output = u8;

    fn index(&self, ij: [u32; 2]) -> &u8 { &self.data[self.layout.offset(ij).unwrap()] }
}

impl<'a> Index<[u32; 2]> for TileMut<'a> {
    type Output = u8;

    fn index(&self, ij: [u32; 2]) -> &u8 { &self.data[self.layout.offset(ij).unwrap()] }
}

impl<'a> IndexMut<[u32; 2]> for TileMut<'a> {
    fn index_mut(&mut self, ij: [u32; 2]) -> &mut u8 { &mut self.data[self.layout.offset(ij).unwrap()] }
}

