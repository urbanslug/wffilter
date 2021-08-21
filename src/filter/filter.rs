use coitrees::{COITree, IntervalNode};
use rayon::prelude::*;
use std::collections::HashSet;
use wflambda_rs as wflambda;

use super::types::*;
use crate::paf;
use crate::types::AppConfig;

pub fn generate_segments(tlen: usize, qlen: usize, config: &AppConfig) -> Vec<Segment> {
    let segment_length: usize = config.segment_length;
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

        if config.step {
            start += step_size;
        } else {
            start += segment_length;
        }
    }

    segments
}

// TODO: remove
/*
pub fn _build_index() -> Index {

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
*/
#[allow(unused_mut, unused_variables)]
pub fn _run_align(
    segments: &Vec<Segment>,
    index: &Index,
) -> (HashSet<QueryResult>, HashSet<QueryResult>) {
    let query_index: &COITree<PafMetadata, u32> = &index.query_index;
    let target_index: &COITree<PafMetadata, u32> = &index.target_index;

    // TODO: remove
    let mut query_lines: HashSet<QueryResult> = HashSet::new();
    let mut text_lines: HashSet<QueryResult> = HashSet::new();

    // The regions in which the global alignment passed through
    let mut overlaps: HashSet<QueryResult> = HashSet::new();
    let mut foobar: HashSet<MatchRegion> = HashSet::new();
    let mut matching_regions: Vec<MatchRegion> = Vec::new();

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

            let mut prelim_query_lines: HashSet<QueryResult> = HashSet::new();
            let mut prelim_text_lines: HashSet<QueryResult> = HashSet::new();

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

                prelim_text_lines.insert(res);
                // text_lines.insert(res);
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

                prelim_query_lines.insert(res);
                //query_lines.insert(res);
            };

            // TODO: do this once
            target_index.query(h_start, h_stop, save_matching_targets);
            target_index.query(h_start, h_stop, save_matching_queries);

            // was there a match or not?

            // TODO: use some kind of fold
            let intersect = prelim_text_lines
                .intersection(&prelim_query_lines)
                .map(|x: &QueryResult| {
                    let x = x.clone();
                    overlaps.insert(x);
                    foobar.insert(MatchRegion {
                        query_start: x.segment_qstart as usize,
                        query_stop: x.segment_qstop as usize,
                        text_start: x.segment_tstop as usize,
                        text_stop: x.segment_tstop as usize,
                    });

                    x
                })
                .collect::<HashSet<QueryResult>>();

            // the intersection is not empty
            !intersect.is_empty()
        };

        let mut traceback_lambda =
            |(q_start, q_stop): (i32, i32), (t_start, t_stop): (i32, i32)| {
                let region = MatchRegion {
                    query_start: q_start as usize,
                    query_stop: q_stop as usize,
                    text_start: t_start as usize,
                    text_stop: t_stop as usize,
                };

                matching_regions.push(region);
            };

        let wflambda_config = wflambda::Config {
            adapt: false,
            segment_length: 1_000,
            step_size: 500,
            thread_count: 36,
            verbosity: 0,
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

    let filtered: Vec<&MatchRegion> = matching_regions
        .iter()
        .filter(|x| foobar.contains(x))
        .collect();

    // filtered.iter().for_each(|x| eprintln!("{:?}", x));
    matching_regions.iter().for_each(|x| eprintln!("{:?}", x));
    eprintln!("---------------------");
    foobar.iter().for_each(|x| eprintln!("{:?}", x));

    (text_lines, query_lines)
}

pub fn filter(_index: &Index, paf: &paf::PAF, config: &AppConfig) {
    let alignment_pairs: HashSet<paf::AlignmentPair> = paf.get_unique_alignments();
    let metadata = paf.get_metadata();

    let _alignment_pairs: Vec<((&str, u32), (&str, u32))> = alignment_pairs
        .par_iter()
        .map(|alignment_pair: &paf::AlignmentPair| {
            let target_name = &alignment_pair.target_name[..];
            let query_name = &alignment_pair.query_name[..];

            let tlen = metadata.get(target_name).unwrap().length;
            let qlen = metadata.get(query_name).unwrap().length;

            let _segments = generate_segments(tlen as usize, qlen as usize, config);

            ((target_name, tlen), (query_name, qlen))
        })
        .collect();
}
