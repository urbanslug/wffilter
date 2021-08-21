use coitrees::COITree;
use std::fmt;
use std::hash::{Hash, Hasher};

// A Segment is a text, query pair of lo, hi or start, stop of the filter
#[allow(dead_code)]
pub type Segment = ((usize, usize), (usize, usize));
// pub type Length = u32;

#[derive(Clone)]
pub struct PafMetadata {
    pub name: String,
    pub line_num: u32, // the line of the alignment in the PAF file
}

pub struct Index {
    pub target_index: COITree<PafMetadata, u32>,
    pub query_index: COITree<PafMetadata, u32>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct MatchRegion {
    pub query_start: usize,
    pub query_stop: usize,
    pub text_start: usize,
    pub text_stop: usize,
}

#[derive(Debug, Copy, Clone)]
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

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Strand {
    Forward,
    Reverse,
}

impl fmt::Display for Strand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let x: char = if *self == Strand::Forward { '+' } else { '-' };
        f.write_str(&x.to_string())
    }
}

// start, stop, line number, name
#[derive(PartialEq, Debug)]
pub struct Interval(pub u32, pub u32, pub usize, pub String);

#[derive(PartialEq, Clone, Copy)]
pub enum SequenceType {
    Target,
    Query,
}

impl fmt::Display for SequenceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            SequenceType::Target => write!(f, "Target"),
            SequenceType::Query => write!(f, "Query"),
        }
    }
}
