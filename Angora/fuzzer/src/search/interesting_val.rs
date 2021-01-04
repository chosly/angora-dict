// From AFL
use std::{
    fs,
    io::{self, prelude::*, BufReader},
    collections::HashMap,
};

use indexmap::IndexMap;

use angora_common::tag::TagSeg;

static INTERESTING_V0: [u64; 1] = [0];

static INTERESTING_V8: [u64; 9] = [
    128, /* Overflow signed 8-bit when decremented  */
    255, /* -1                                       */
    0,   /*                                         */
    1,   /*                                         */
    16,  /* One-off with common buffer size         */
    32,  /* One-off with common buffer size         */
    64,  /* One-off with common buffer size         */
    100, /* One-off with common buffer size         */
    127, /* Overflow signed 8-bit when incremented  */
];

static INTERESTING_V16: [u64; 19] = [
    65408, /* Overflow signed 8-bit when decremented  */
    65535, /*                                         */
    0,     /*                                         */
    1,     /*                                         */
    16,    /* One-off with common buffer size         */
    32,    /* One-off with common buffer size         */
    64,    /* One-off with common buffer size         */
    100,   /* One-off with common buffer size         */
    127,   /* Overflow signed 8-bit when incremented  */
    32768, /* Overflow signed 16-bit when decremented */
    65407, /* Overflow signed 8-bit                   */
    128,   /* Overflow signed 8-bit                   */
    255,   /* Overflow unsig 8-bit when incremented   */
    256,   /* Overflow unsig 8-bit                    */
    512,   /* One-off with common buffer size         */
    1000,  /* One-off with common buffer size         */
    1024,  /* One-off with common buffer size         */
    4096,  /* One-off with common buffer size         */
    32767, /* Overflow signed 16-bit when incremented */
];

static INTERESTING_V32: [u64; 27] = [
    4294967168, /* Overflow signed 8-bit when decremented  */
    4294967295, /*                                         */
    0,          /*                                         */
    1,          /*                                         */
    16,         /* One-off with common buffer size         */
    32,         /* One-off with common buffer size         */
    64,         /* One-off with common buffer size         */
    100,        /* One-off with common buffer size         */
    127,        /* Overflow signed 8-bit when incremented  */
    4294934428, /* Overflow signed 16-bit when decremented */
    4294967167, /* Overflow signed 8-bit                   */
    128,        /* Overflow signed 8-bit                   */
    255,        /* Overflow unsig 8-bit when incremented   */
    256,        /* Overflow unsig 8-bit                    */
    512,        /* One-off with common buffer size         */
    1000,       /* One-off with common buffer size         */
    1024,       /* One-off with common buffer size         */
    4096,       /* One-off with common buffer size         */
    32767,      /* Overflow signed 16-bit when incremented */
    2147483648, /* Overflow signed 32-bit when decremented */
    4194304250, /* Large negative number (endian-agnostic) */
    4194304250, /* Overflow signed 16-bit                  */
    32768,      /* Overflow signed 16-bit                  */
    65535,      /* Overflow unsig 16-bit when incremented  */
    65536,      /* Overflow unsig 16 bit                   */
    100663045,  /* Large positive number (endian-agnostic) */
    2147483647, /* Overflow signed 32-bit when incremented */
];

pub fn get_interesting_bytes<'a>(width: usize) -> &'a [u64] {
    match width {
        1 => &INTERESTING_V8,
        2 => &INTERESTING_V16,
        4 | 8 => &INTERESTING_V32,
        _ => {
            &INTERESTING_V0
            // do nothing
        },
    }
}

#[derive(Debug, Default, Clone)]
pub struct SCond {
    pub cmpid: u32,
    pub offsets: Vec<TagSeg>,
}

impl SCond {
    pub fn new(cmpid: u32, offsets: Vec<TagSeg>) -> Self {
        Self {
            cmpid: cmpid,
            offsets: offsets,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Word (pub String);

impl Word {
    pub fn new(data: String) -> Self {
        let tmp = data.clone();
        Self(tmp)
    }

    pub fn len(&self) -> usize {
        self.0.as_bytes().len()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0.as_bytes()
    }
}

#[derive(Default, Clone, Debug)]
pub struct Dict (pub IndexMap<usize, Vec<Vec<u8>>>);

impl Dict {
    pub fn filter(&mut self, conds: Vec<SCond>, buf: Vec<u8>) {
        for cond in conds {
            let mut words: Vec<Vec<u8>> = Vec::new();
            let mut i = 0;
            let mut min: usize = std::usize::MAX;
            let mut max: usize = std::usize::MIN;
            loop {
                if i == cond.offsets.len() { break; }

                let idx = i;
                let mut end = cond.offsets[i].end;

                for j in i+1..cond.offsets.len() {
                    if end != cond.offsets[j].begin { break; }
                    end = cond.offsets[j].end;
                    i = j;
                }
                words.push(buf[(cond.offsets[idx].begin as usize)..(cond.offsets[i].end as usize)].to_vec());
                i += 1;
            }

            for offset in cond.offsets {
                min = if min > offset.begin as usize { offset.begin as usize } else { min };
                max = if max < offset.end as usize { offset.end as usize } else { max };
            }
            if min < max { words.push(buf[min..max].to_vec()); }

            for word in words {
                let len = word.len();
                if let Some(x) = self.0.get_mut(&len) {
                    if !x.contains(&word) {
                        x.push(word);
                    }
                }
                else {
                    self.0.insert(len, vec![word]);
                    self.0.sort_keys();
                }
            }
        }
    }

    pub fn get_list(&self, idx: usize) -> Vec<Vec<u8>> {
        match &self.0.get_index(idx) {
            Some(x) => { x.1.to_vec() }
            _ => { warn!("Can't find the dictionary"); vec![] }
        }
    }

    pub fn is_empty(&self) -> bool {
        return self.0.is_empty()
    }

    pub fn get_len(&self) -> String {
        let mut arr = [0; 8];
        for (l, k) in self.0.iter() {
            // [1], [2], [3], [4, 7], [8, 15], [16, 31], [32, 127], [128, infinity]
            if *l == 1 {
                arr[0] += k.len();
            } else if *l == 2 {
                arr[1] += k.len();
            } else if *l == 4 {
                arr[2] += k.len();
            } else if *l >= 4 && *l <= 7 {
                arr[3] += k.len();
            } else if *l >= 8 && *l <= 15 {
                arr[4] += k.len();
            } else if *l >= 16 && *l <= 31 {
                arr[5] += k.len();
            } else if *l >= 32 && *l <= 127 {
                arr[6] += k.len();
            } else if *l >= 128 {
                arr[7] += k.len();
            }
        }
        let mut result = String::new();
        for i in arr.iter() {
            result += &i.to_string();
            result += ",";
        }
        result
    }
}
