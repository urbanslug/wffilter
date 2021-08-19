use coitrees::{COITree, IntervalNode};
use std::collections::HashSet;

use std::fmt::Debug;

use super::types::*;

use wflambda_rs as wflambda;

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

#[allow(unused_mut, unused_variables)]
pub fn run_align(
    segments: &Vec<Segment>,
    index: &Index,
) -> (HashSet<QueryResult>, HashSet<QueryResult>) {
    let query_index: &COITree<PafMetadata, u32> = &index.query_index;
    let target_index: &COITree<PafMetadata, u32> = &index.target_index;

    // The regions in which the global alignment passed through
    let mut query_lines: HashSet<QueryResult> = HashSet::new();
    let mut text_lines: HashSet<QueryResult> = HashSet::new();

    let wflambda_config = wflambda::Config {
        adapt: true,
        segment_length: 1_000,
        step_size: 500,
        thread_count: 36,
        verbosity: 0,
    };

    // TODO: should this be concurrent?
    for segment in segments {
        let ((tstart, tstop), (qstart, qstop)) = *segment;

        let mut match_lambda = |v: &mut usize, h: &mut usize| -> bool {
            // We are matching segments that are the seize of segment_length
            // add v and h by qstart and tstart to make up for the offset created by the segment
            // we are basically doing position in the segment + position of the segment
            let v_start = (*v + qstart) as i32;
            let h_start = (*h + tstart) as i32;

            let v_stop = (*v + qstop) as i32;
            let h_stop = (*h + tstop) as i32;

            *v = v_stop as usize;
            *h = h_stop as usize;

            // FIXME: use `query`. Not using `query` for now because of the type checker
            // https://docs.rs/coitrees/0.2.1/coitrees/struct.COITree.html#method.query

            let save_matching_targets = |i: &IntervalNode<PafMetadata, u32>| {
                let res = QueryResult {
                    line: i.metadata.line_num,

                    sequence_start: i.first,
                    sequence_stop: i.last,

                    segment_qstart: qstart,
                    segment_qstop: qstop,
                    segment_tstart: tstart,
                    segment_tstop: tstop,
                };

                text_lines.insert(res);
            };

            let save_matching_queries = |i: &IntervalNode<PafMetadata, u32>| {
                let res = QueryResult {
                    line: i.metadata.line_num,

                    sequence_start: i.first,
                    sequence_stop: i.last,

                    segment_qstart: qstart,
                    segment_qstop: qstop,
                    segment_tstart: tstart,
                    segment_tstop: tstop,
                };

                query_lines.insert(res);
            };

            // TODO: do this once
            target_index.query(h_start, h_stop, save_matching_targets);
            target_index.query(h_start, h_stop, save_matching_queries);

            target_index.query_count(h_start, h_stop) > 0 && query_index.query_count(v_start, v_stop) > 0
        };

        let mut matching_regions: Vec<wflambda::MatchRegion> = Vec::new();

        let mut traceback_lambda = |(q_start, q_stop): (i32, i32), (t_start, t_stop): (i32, i32)| {
            matching_regions.push(wflambda::MatchRegion {
                query_start: q_start,
                query_stop: q_stop,
                text_start: t_start,
                text_stop: t_stop,
            });
        };

        let tlen = tstop - tstart;
        let qlen = qstop - qstart;

        wflambda::wf_align(
            tlen,
            qlen,
            &wflambda_config,
            &mut match_lambda,
            &mut traceback_lambda,
        );
    }

    text_lines.iter().for_each(|x| eprintln!("{}", x));

    
    eprintln!("-------------------------------");

    query_lines.iter().for_each(|x| eprintln!("{}", x));
    // eprintln!("{:?}", query_lines);

    (text_lines, query_lines)
}
