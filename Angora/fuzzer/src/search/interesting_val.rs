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
pub struct Dict (pub IndexMap<u32, Vec<Vec<u8>>>);

impl Dict {
    pub fn filter(&mut self, conds: Vec<SCond>, buf: Vec<u8>) {
        info!("before: {:?}", self.0);
        for cond in conds {
            let mut words: Vec<Vec<u8>> = Vec::new();
            for tag in cond.offsets {
                words.push(buf[(tag.begin as usize)..(tag.end as usize + 1)].to_vec());
            }
            /*
            for word in words.clone() {
                info!("{}", String::from_utf8_lossy(&word));
            }
            */
            if let Some(x) = self.0.get_mut(&cond.cmpid) {
                for word in words {
                    if !x.contains(&word) {
                        x.push(word);
                    }
                }
            }
            else {
                self.0.insert(cond.cmpid, words);
            }
        }
        info!("after: {:?}", self.0);
    }
}
/*pub struct Dict (pub IndexMap<usize, Vec<Word>>);

impl Dict {
    pub fn parse_dict(file: fs::File) -> Dict {
        let mut dict: Dict = Default::default();
        let reader = BufReader::new(file);

        for r in reader.lines() {
            let line = r.unwrap();
            if line == "\n" {
                continue;
            }

            let mut first_quote: i32 = -1;
            let mut second_quote: i32 = -1;
            for (i, ch) in line.chars().enumerate() {
                if ch == '\"' {
                    if first_quote == -1 {
                        first_quote = i as i32;
                    }
                    else {
                        second_quote = i as i32;
                    }
                }
            }
            if first_quote > -1 && second_quote > -1 {
                let data = line[first_quote as usize + 1..second_quote as usize].to_string();
                let word = Word::new(data);
                let len = word.0.len();
                if let Some(x) = dict.0.get_mut(&len) {
                    x.push(word);
                }
                else {
                    dict.0.insert(len, vec![word]);
                }
            }
        }

        dict.0.sort_keys();
        dict
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get_list(&self, idx: usize) -> Vec<Word> {
        match &self.0.get_index(idx) {
            Some(x) => { x.1.to_vec() }
            _ => { warn!("Can't find the dictionary"); vec![] }
        }
    }

    pub fn is_empty(&self) -> bool {
        return self.0.is_empty()
    }
}*/
