use coitrees::{COITree, IntervalNode};
use std::collections::HashSet;
use std::fmt;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
// use rayon::prelude::*;

// Use overlapping segments
const STEP: bool = true;

pub fn pretty_print_vec<T>(v: &Vec<T>, width: usize)
where
    T: Debug,
{
    for i in 0..v.len() {
        eprint!("{:?} ", v[i]);
        if (i + 1) % width == 0 {
            eprintln!();
        }
    }
}

#[derive(Clone, Copy)]
struct PafMetadata {
    pub line_num: u32, // the line of the alignment in the PAF file
}

struct Index {
    pub target_index: COITree<PafMetadata, u32>,
    pub query_index: COITree<PafMetadata, u32>,
}

// A Segment here a text, query pair of lo, hi or start, stop of the running our check
type Segment = ((usize, usize), (usize, usize));

fn generate_segments(tlen: usize, qlen: usize, segment_length: usize) -> Vec<Segment> {
    // let's generate overlapping segments to make it more realistic
    let step_size = (segment_length as f64 / 2_f64).floor() as usize;

    let mut segments: Vec<((usize, usize), (usize, usize))> = Vec::new();

    let mut start: usize = 0;
    let mut stop: usize = 0;

    while stop < std::cmp::min(tlen, qlen) {
        stop = start + segment_length;
        if stop >= std::cmp::min(tlen, qlen) {
            segments.push(((start, tlen), (start, qlen)));
        } else {
            segments.push(((start, stop), (start, stop)));
        }

        if STEP {
            start += step_size;
        } else {
            start += segment_length;
        }
    }

    segments
}

fn build_index() -> Index {
    /*
    ---
    PAF
    ---
    1     10     3     13    +    5M1I3M
    14    31     36    54    +    10M1X5M

    --------
    Breakdown
    --------
    We have matches running like so
    Query matces
    paf row 1
    1 5
    6 9

    paf row 2
    14 24
    26 31

    Target matches
    paf row 1
    3 8
    7 10

    paf row 2
    36 46
    48 53
    */

    let query_interval_nodes: Vec<IntervalNode<PafMetadata, u32>> = vec![
        IntervalNode::new(1, 5, PafMetadata { line_num: 1 }),
        IntervalNode::new(6, 9, PafMetadata { line_num: 1 }),
        IntervalNode::new(14, 24, PafMetadata { line_num: 2 }),
        IntervalNode::new(26, 31, PafMetadata { line_num: 2 }),
    ];

    let target_interval_nodes: Vec<IntervalNode<PafMetadata, u32>> = vec![
        IntervalNode::new(3, 8, PafMetadata { line_num: 1 }),
        IntervalNode::new(7, 10, PafMetadata { line_num: 1 }),
        IntervalNode::new(36, 46, PafMetadata { line_num: 2 }),
        IntervalNode::new(48, 53, PafMetadata { line_num: 2 }),
    ];

    Index {
        query_index: COITree::new(query_interval_nodes),
        target_index: COITree::new(target_interval_nodes),
    }
}

#[derive(Debug)]
struct QueryResult {
    line: u32,

    start: i32,
    stop: i32,

    segment_qstart: usize,
    segment_qstop: usize,
    segment_tstart: usize,
    segment_tstop: usize,
}

impl PartialEq for QueryResult {
    fn eq(&self, other: &Self) -> bool {
        self.line == other.line
            && self.segment_tstart == other.segment_tstart
            && self.segment_tstop == other.segment_tstop
            && self.segment_qstart == other.segment_qstart
            && self.segment_qstop == other.segment_qstop
    }
}
impl Eq for QueryResult {}

impl Hash for QueryResult {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.line.hash(state);
        self.segment_tstart.hash(state);
        self.segment_tstop.hash(state);
        self.segment_qstart.hash(state);
        self.segment_qstop.hash(state);
    }
}

impl fmt::Display for QueryResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let aln = format!(
            "QueryResult {{\n\
             \tline: {}\n\
             \tstart: {}\n\
             \tstop: {}\n\
             \tsegment_tstart: {}\n\
             \tsegment_tstart: {}\n\
             \tsegment_qstart: {}\n\
             \tsegment_qstop: {}\n\
             }}",
            self.line,
            self.start,
            self.stop,
            self.segment_tstart,
            self.segment_tstop,
            self.segment_qstart,
            self.segment_qstop,
        );

        write!(f, "{}", aln)
    }
}

fn main() {
    let tlen: usize = 60 as usize;
    let qlen: usize = 60 as usize;
    let segment_length: usize = 5;
    // generate segments
    let segments = generate_segments(tlen, qlen, segment_length);
    // println!("The segments are:\n {:?}\n\n", segments);
    // eprintln!("The segments are");
    // utils::pretty_print_vec(&segments, 3);
    // eprintln!();

    // build Index
    let index = build_index();
    let query_index = index.query_index;
    let target_index = index.target_index;

    let mut query_lines: HashSet<QueryResult> = HashSet::new();
    let mut text_lines: HashSet<QueryResult> = HashSet::new();

    segments
        .into_iter() // we can make this concurrent
        .for_each(
            |((tstart, tstop), (qstart, qstop)): ((usize, usize), (usize, usize))| {
                let foo = |i: &IntervalNode<PafMetadata, u32>| {
                    let res = QueryResult {
                        line: i.metadata.line_num,

                        start: i.first,
                        stop: i.last,

                        segment_qstart: qstart,
                        segment_qstop: qstop,
                        segment_tstart: tstart,
                        segment_tstop: tstop,
                    };

                    text_lines.insert(res);
                };
                let baz = |i: &IntervalNode<PafMetadata, u32>| {
                    let res = QueryResult {
                        line: i.metadata.line_num,

                        start: i.first,
                        stop: i.last,

                        segment_qstart: qstart,
                        segment_qstop: qstop,
                        segment_tstart: tstart,
                        segment_tstop: tstop,
                    };

                    query_lines.insert(res);
                };

                target_index.query(tstart as i32, tstop as i32, foo);
                query_index.query(qstart as i32, qstop as i32, baz);
            },
        );

    // query the index
    // are there matches in this segment?
    let overlap: Vec<&QueryResult> = text_lines.intersection(&query_lines).collect();
    eprintln!("Number of results {}", overlap.len());

    for x in overlap.iter() {
        println!("{}", x);
    }
}
