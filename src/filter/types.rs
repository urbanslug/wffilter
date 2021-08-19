use coitrees::{COITree};
use std::hash::{Hash, Hasher};
use std::fmt;

// A Segment is a text, query pair of lo, hi or start, stop of the filter
pub type Segment = ((usize, usize), (usize, usize));


#[derive(Clone, Copy)]
pub struct PafMetadata {
    pub line_num: u32, // the line of the alignment in the PAF file
}

pub struct Index {
    pub target_index: COITree<PafMetadata, u32>,
    pub query_index: COITree<PafMetadata, u32>,
}

#[derive(Debug)]
pub struct QueryResult {
    pub line: u32, // PAF record ID the line in the PAF file from which we got this result

    // the start and end positions on the sequence itself
    pub sequence_start: i32,
    pub sequence_stop: i32,

    pub segment_qstart: usize,
    pub segment_qstop: usize,
    pub segment_tstart: usize,
    pub segment_tstop: usize,
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
             \tsequence_start: {}\n\
             \tsequence_stop: {}\n\
             \tsegment_tstart: {}\n\
             \tsegment_tstart: {}\n\
             \tsegment_qstart: {}\n\
             \tsegment_qstop: {}\n\
             }}",
            self.line,
            self.sequence_start,
            self.sequence_stop,
            self.segment_tstart,
            self.segment_tstop,
            self.segment_qstart,
            self.segment_qstop,
        );

        write!(f, "{}", aln)
    }
}
