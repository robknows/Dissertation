// Decomposed cracking
// Underswap RLE, recognitive compression

// uses map! macro.

use column::Column;
use column::IntCol;
use cracker_index::AVLCrackerIndex;
use std::collections::HashMap;
use std::slice::Iter;

#[derive(Clone)]
pub struct UnderswapRLETable {
    pub count: usize,
    pub crk_col: IntCol,
    pub columns: HashMap<String, IntCol>,
    pub dbg_switch: bool,
}

impl UnderswapRLETable {
    pub fn new() -> UnderswapRLETable {
        UnderswapRLETable {
            count: 0,
            crk_col: IntCol::empty(),
            columns: HashMap::new(),
            dbg_switch: false,
        }
    }

    pub fn print_cols(&self) {
        println!("crk: {:?}", self.crk_col.crk);
        println!("ofs: {:?}", self.crk_col.ofs);
        println!("rls: {:?}", self.crk_col.run_lengths);
        for (name, col) in self.columns.clone() {
            println!("{}: {:?}", name, col.v);
        }
    }

    pub fn print_rl_crk(&self) {
        println!("crk: {:?}", self.crk_col.crk);
        println!("rls: {:?}", self.crk_col.run_lengths);
    }

    pub fn print_rl_crk_range(&self, lower: usize, upper: usize) {
        print!("crk: [{}", self.crk_col.crk[lower]);
        for i in (lower + 1)..upper {
            print!(", {}", self.crk_col.crk[i])
        }
        print!("]\n");
        print!("rls: [{}", self.crk_col.run_lengths[lower]);
        for i in (lower + 1)..upper {
            print!(", {}", self.crk_col.run_lengths[i])
        }
        print!("]\n");
    }

    pub fn print_crk(&self) {
        println!("crk: {:?}", self.crk_col.crk);
    }

    pub fn new_columns(&mut self, col_names: Vec<&str>) {
        for col in col_names {
            self.columns.insert(col.to_string(), IntCol::empty());
        }
    }

    pub fn set_crk_col(&mut self, col_name: &str) {
        match self.columns.get(&(col_name.to_string())) {
            Some(ref c) => {
                self.crk_col.v = c.v.clone();
                self.crk_col.crk = c.v.clone();
                self.crk_col.base_idx = (0..self.count).collect();
                self.crk_col.run_lengths = vec![1;self.count];
            },
            None => panic!("set_crk_col: no such col"),
        };
    }

    // TODO: Improve exception handling in this function
    pub fn insert(&mut self, new_values: &mut HashMap<&str, Vec<i64>>) {
        let mut n_new_tuples = 0;
        for (key, val) in self.columns.iter_mut() {
            let new_elements = new_values.get_mut(&*(key.clone())).unwrap();
            let n = new_elements.len();
            if n_new_tuples == 0 || n_new_tuples == n {
                val.v.append(new_elements);
                n_new_tuples = n;
            } else {
                panic!("insert: new_values has vectors of differing lengths");
            }
        }
        self.count += n_new_tuples;
    }

    pub fn get_col(&self, col: &str) -> &IntCol {
        self.columns.get(&(col.to_string())).unwrap()
    }

    pub fn get_indices(&self, indices: Iter<usize>) -> UnderswapRLETable {
        let mut selection: HashMap<String, IntCol> = HashMap::new();
        for (name, col) in &self.columns {
            let mut v_buffer = Vec::with_capacity(indices.len());
            for &i in indices.clone() {
                v_buffer.push(col.v[i]);
            }
            let mut c_buffer = IntCol::empty();
            c_buffer.v = v_buffer;
            selection.insert(name.clone(), c_buffer);
        }

        let mut t = UnderswapRLETable::new();
        t.columns = selection;
        t.count = indices.len();

        let mut indexed_crk_v = Vec::with_capacity(indices.len());
        if self.crk_col.crk.len() > 0 {
            let mut indexed_crk_col = Vec::with_capacity(indices.len());
            for &i in indices.clone() {
                indexed_crk_col.push(self.crk_col.crk[i]);
                indexed_crk_v.push(self.crk_col.v[i]);
            }
            t.crk_col.crk = indexed_crk_col;
            t.crk_col.crk_idx = AVLCrackerIndex::new();
        } else {
            for &i in indices.clone() {
                indexed_crk_v.push(self.crk_col.v[i]);
            }
        }
        t.crk_col.v = indexed_crk_v;
        t
    }

    pub fn get_values(&self, indices: Iter<usize>, col: &str) -> Vec<i64> {
        let mut buf = Vec::new();
        for &i in indices {
            buf.push(self.columns[&col.to_string()].v[i]);
        }
        buf
    }

    pub fn rearrange(&mut self, indices: Iter<usize>) {
        for col in self.columns.values_mut() {
            col.rearrange(indices.clone());
        }
        self.crk_col.rearrange(indices.clone());
    }

    // Returns the elements of T where the cracker columns's value equals X
    pub fn cracker_select_specific(&mut self, x: i64, col: &str) -> Vec<i64> {
        // Setup
        let mut p_low  = self.crk_col.crk_idx.lower_bound(&x).unwrap_or(0);
        if p_low == self.count {
            return vec![];
        }
        let mut p_high = self.crk_col.crk_idx.upper_bound(&(x + 1)).unwrap_or(self.count) - 1;
        if p_high + 1 == 0 { return vec![] };

        // Tighten
        while self.crk_col.crk[p_low] < x && p_low < p_high {
            let mut rl = self.crk_col.run_lengths[p_low];

            while p_low + rl >= self.count && p_low + 1 < self.count { // Evade overflow.
                p_low += 1;
                rl = self.crk_col.run_lengths[p_low];
            }
            if p_low + rl >= self.count {
                break;
            }

            if self.crk_col.crk[p_low + rl] == self.crk_col.crk[p_low] {
                while self.crk_col.crk[p_low + rl] == self.crk_col.crk[p_low] {
                    let inc = self.crk_col.run_lengths[p_low + rl];
                    if p_low + rl + inc >= p_high {
                        break;
                    }
                    rl += inc;
                }
                self.crk_col.run_lengths[p_low]          = rl;
                self.crk_col.run_lengths[p_low + rl - 1] = rl;
            }
            p_low += rl;
        }

        while self.crk_col.crk[p_high] > x && p_high > p_low {
            let mut rl = self.crk_col.run_lengths[p_high];
            if self.crk_col.crk[p_high - rl] == self.crk_col.crk[p_high] {
                while self.crk_col.crk[p_high - rl] == self.crk_col.crk[p_high] {
                    let inc = self.crk_col.run_lengths[p_high - rl];
                    if p_high < rl + inc {
                        break;
                    } else if p_high - (rl + inc) < p_low {
                        break;
                    }
                    rl += inc;
                }
                self.crk_col.run_lengths[p_high]            = rl;
                self.crk_col.run_lengths[(p_high - rl) + 1] = rl;
            }
            p_high -= rl;
        }

        if p_low == p_high {
            if self.crk_col.crk[p_low] == x {
                return self.get_values(self.crk_col.base_idx[p_low..(p_low + 1)].iter(), col);
            } else {
                return vec![];
            }
        }

        // Scan
        let mut p_itr = p_low.clone();
        while p_itr <= p_high {
            if self.crk_col.crk[p_itr] < x {
                let rl_itr = self.crk_col.run_lengths[p_itr];
                let rl_low = self.crk_col.run_lengths[p_low];

                if rl_itr == rl_low {
                    self.crk_col.swap_range(rl_itr, p_low, p_itr);
                    p_low += rl_itr;
                } else {
                    // Combine the runs
                    let rl_low = p_itr - p_low;

                    if rl_itr > rl_low { // |S|BIG|
                        // Set up L-run
                        self.crk_col.run_lengths[p_low]     = rl_low;
                        self.crk_col.run_lengths[p_itr - 1] = rl_low;
                        // Prepare I-run for swap
                        self.crk_col.run_lengths[p_itr + rl_itr - rl_low]     = rl_itr;
                        self.crk_col.run_lengths[p_itr + rl_itr - rl_low - 1] = rl_itr;
                        self.crk_col.swap_range(rl_low, p_low, p_itr + rl_itr - rl_low);
                        p_low += rl_itr;
                        p_itr += rl_itr - rl_low;
                    } else if rl_itr < rl_low { // |BIG|S|
                        // Prepare L-run for swap
                        self.crk_col.run_lengths[p_low + rl_itr]     = rl_low;
                        self.crk_col.run_lengths[p_low + rl_itr - 1] = rl_low;
                        self.crk_col.swap_range(rl_itr, p_low, p_itr);
                        p_low += rl_itr;
                        p_itr += rl_itr;
                    } else { // |EQ|EQ|
                        // Set up L-run
                        self.crk_col.run_lengths[p_low]     = rl_low;
                        self.crk_col.run_lengths[p_itr - 1] = rl_low;
                        self.crk_col.swap_range(rl_itr, p_low, p_itr);
                        p_low += rl_itr;
                    }
                }

                while self.crk_col.crk[p_low] < x && p_low < p_high {
                    let mut rl = self.crk_col.run_lengths[p_low];

                    while p_low + rl >= self.count && p_low + 1 < self.count { // Evade overflow.
                        p_low += 1;
                        rl = self.crk_col.run_lengths[p_low];
                    }
                    if p_low + rl >= self.count {
                        break;
                    }

                    if self.crk_col.crk[p_low + rl] == self.crk_col.crk[p_low] {
                        while self.crk_col.crk[p_low + rl] == self.crk_col.crk[p_low] {
                            let inc = self.crk_col.run_lengths[p_low + rl];
                            if p_low + rl + inc >= p_high {
                                break;
                            }
                            rl += inc;
                        }
                        self.crk_col.run_lengths[p_low]          = rl;
                        self.crk_col.run_lengths[p_low + rl - 1] = rl;
                    }
                    p_low += rl;
                }

                if p_itr < p_low {
                    p_itr = p_low.clone();
                }
            } else if self.crk_col.crk[p_itr] > x {
                let rl_itr = self.crk_col.run_lengths[p_itr];
                let rl_high = self.crk_col.run_lengths[p_high];

                let n_swaps = if rl_itr == rl_high {
                    rl_itr
                } else if rl_itr < rl_high {
                        self.crk_col.run_lengths[p_high - rl_high + 1] -= rl_itr;
                        self.crk_col.run_lengths[p_high - rl_itr]       = self.crk_col.run_lengths[p_high - rl_high + 1];
                        self.crk_col.run_lengths[p_high]                = rl_itr;
                        self.crk_col.run_lengths[p_high - rl_itr + 1]   = rl_itr;
                        rl_itr
                 } else {
                        self.crk_col.run_lengths[p_itr + rl_itr - 1] -= rl_high;
                        self.crk_col.run_lengths[p_itr + rl_high]     = self.crk_col.run_lengths[p_itr + rl_itr - 1];
                        self.crk_col.run_lengths[p_itr]               = rl_high;
                        self.crk_col.run_lengths[p_itr + rl_high - 1] = rl_high;
                        rl_high
                };

                self.crk_col.swap_range(n_swaps, p_itr, p_high - n_swaps + 1);

                p_high -= n_swaps;

                while self.crk_col.crk[p_high] > x && p_high > p_low {
                    let mut rl = self.crk_col.run_lengths[p_high];
                    if self.crk_col.crk[p_high - rl] == self.crk_col.crk[p_high] {
                        while self.crk_col.crk[p_high - rl] == self.crk_col.crk[p_high] {
                            let inc = self.crk_col.run_lengths[p_high - rl];
                            if p_high < rl + inc {
                                break;
                            } else if p_high - (rl + inc) < p_low {
                                break;
                            }
                            rl += inc;
                        }
                        self.crk_col.run_lengths[p_high]            = rl;
                        self.crk_col.run_lengths[(p_high - rl) + 1] = rl;
                    }
                    p_high -= rl;
                }
            } else {
                p_itr += self.crk_col.run_lengths[p_itr];
            }
        }

        // If nothing is selected, then return nothing
        if p_high < p_low {
            return vec![];
        }

        // Memo
        // Combine the fragment into a run
        self.crk_col.run_lengths[p_low]  = p_high - p_low + 1;
        self.crk_col.run_lengths[p_high] = p_high - p_low + 1;
        //Store in cracker index
        self.crk_col.crk_idx.insert(x, p_low);
        self.crk_col.crk_idx.insert(x + 1, p_high + 1);
        self.get_values(self.crk_col.base_idx[p_low..(p_high + 1)].iter(), col)
    }

    // Counts the places where a given column equals a given value
    pub fn count_col_eq(&self, col: &str, eq: i64) -> i64 {
        self.get_col(col).v.iter().map(|&x|(x==eq)as i64).fold(0,|sum,x|sum+x) as i64
    }
}

// Returns an adjacency list built from the two vectors of adjacent nodes.
pub fn from_adjacency_vectors(src_node: Vec<i64>, dst_node: Vec<i64>, crk: &str) -> UnderswapRLETable {
    let mut adjacency_list = UnderswapRLETable::new();
    adjacency_list.new_columns(vec!["src", "dst"]);
    adjacency_list.insert(&mut map!{"src" => src_node, "dst" => dst_node});
    adjacency_list.set_crk_col(crk);
    adjacency_list
}