use coitrees::{COITree, IntervalNode};
use std::collections::HashSet;
use std::fmt;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use crate::wflambda::wflambda;

const STEP: bool = true; // Use overlapping segments

#[allow(dead_code)]
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
pub struct PafMetadata {
    pub line_num: u32, // the line of the alignment in the PAF file
}

pub struct Index {
    pub target_index: COITree<PafMetadata, u32>,
    pub query_index: COITree<PafMetadata, u32>,
}

// A Segment is a text, query pair of lo, hi or start, stop of the filter
type Segment = ((usize, usize), (usize, usize));

pub fn generate_segments(tlen: usize, qlen: usize, segment_length: usize) -> Vec<Segment> {
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

pub fn build_index() -> Index {
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
pub struct QueryResult {
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

#[allow(unused_mut, unused_variables)]
pub fn run_align(
    segments: &Vec<Segment>,
    index: &Index,
) -> (HashSet<QueryResult>, HashSet<QueryResult>) {
    let query_index: &COITree<PafMetadata, u32> = &index.query_index;
    let target_index: &COITree<PafMetadata, u32> = &index.target_index;

    let mut query_lines: HashSet<QueryResult> = HashSet::new();
    let mut text_lines: HashSet<QueryResult> = HashSet::new();

    segments
        .iter() // we can make this concurrent
        .for_each(|i: &((usize, usize), (usize, usize))| {
            let ((tstart, tstop), (qstart, qstop)) = *i;

            let match_lambda = |v: usize, h: usize| -> bool {
                false
            };

            let traceback_lambda = |_: (i32, i32), _: (i32, i32)| {
                
            };

            wflambda::wf_align(
                &match_lambda,
                &traceback_lambda,
                tstop-tstart,
                qstop-qstart,
            );
        });

    (text_lines, query_lines)
}
