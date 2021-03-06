use cracker_index::AVLCrackerIndex;

use std::slice::Iter;

pub trait Column {
    type Item;
    fn empty() -> Self;
    fn rearrange(&mut self, indices: Iter<usize>);
    fn at(self, idx: usize) -> Self::Item;
    fn append(&mut self, values: &mut Vec<Self::Item>);
}

#[derive(Clone)]
pub struct IntCol {
    // Original
    pub v: Vec<i64>,

    // Cracked
    pub crk: Vec<i64>,

    // Cracker index - for a value v, stores the index p such that
    // for all i < p: c[i] < v. That is - Every value before p in the column
    // is less than v.
    pub crk_idx: AVLCrackerIndex,

    // Base index - maintains an index into the base columns of the table for alignment
    // during tuple reconstruction.
    pub base_idx: Vec<usize>,

    // Offset - compressed base_idx for reducing scan time in compactive compression.
    pub ofs: Vec<usize>,

    // Run lengths - stores the run lengths gathered during intra-fragment compression.
    pub run_lengths: Vec<usize>,
}

use std::ptr;

impl IntCol {
    #[inline]
    pub fn swap_range(&mut self, l: usize, a: usize, b: usize) {
        unsafe {
            ptr::swap_nonoverlapping(&mut self.crk[a], &mut self.crk[b], l);
            ptr::swap_nonoverlapping(&mut self.base_idx[a], &mut self.base_idx[b], l);
            ptr::swap_nonoverlapping(&mut self.run_lengths[a], &mut self.run_lengths[b], l);
        }
    }
}

impl Column for IntCol {
    type Item = i64;

    fn empty() -> IntCol {
        IntCol {
            v: Vec::new(),
            crk:Vec::new(),
            crk_idx: AVLCrackerIndex::new(),
            base_idx: Vec::new(),
            ofs: Vec::new(),
            run_lengths: Vec::new(),
        }
    }

    fn rearrange(&mut self, indices: Iter<usize>) {
        let mut replacement_v = Vec::with_capacity(self.v.len());
        for &i in indices.clone() {
            replacement_v.push(self.v[i]);
        }
        self.v = replacement_v;

        // Could be optimised for nested queries
        self.crk = Vec::new();
        self.crk_idx = AVLCrackerIndex::new();
        self.base_idx = Vec::new();
        self.ofs = Vec::new();
        self.run_lengths = Vec::new();
    }

    fn at(self, idx: usize) -> i64 {
        self.v[idx]
    }

    fn append(&mut self, values: &mut Vec<i64>) {
        self.v.append(values);
    }
}