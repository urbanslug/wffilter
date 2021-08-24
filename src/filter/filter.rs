use coitrees::{COITree, IntervalNode};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
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


fn run_aln(
    segments: &Vec<Segment>,
    index: &Index,
    target_name: &str,
    query_name: &str,
    wflambda_config: &wflambda::Config,
    matching_regions: &mut HashSet<QueryResult>,
) {
    let query_index: &COITree<PafMetadata, u32> = &index.query_index;
    let target_index: &COITree<PafMetadata, u32> = &index.target_index;

    for segment in segments {
        let ((tstart, tstop), (qstart, qstop)) = *segment;

        let mut match_matches: HashSet<QueryResult> = HashSet::new();
        let mut traceback_matches: HashSet<QueryResult> = HashSet::new();

        let mut match_lambda = |v: &mut usize, h: &mut usize| -> bool {
            // We are matching segments that are the size of segment_length
            // add v and h by qstart and tstart to make up for the offset created by the segment
            // we are basically doing position in the segment + position of the segment
            let v_start = (*v + qstart) as i32;
            let h_start = (*h + tstart) as i32;

            let v_stop = (*v + qstop) as i32;
            let h_stop = (*h + tstop) as i32;

            *v = v_stop as usize;
            *h = h_stop as usize;

            let mut query_cache: HashSet<QueryResult> = HashSet::new();
            let mut target_cache: HashSet<QueryResult> = HashSet::new();

            let handle_targets = |i: &IntervalNode<PafMetadata, u32>| {
                if i.metadata.name != target_name {
                    return;
                }

                let res = QueryResult {
                    line: i.metadata.line_num,

                    sequence_start: i.first,
                    sequence_stop: i.last,

                    segment_qstart: qstart,
                    segment_qstop: qstop,
                    segment_tstart: tstart,
                    segment_tstop: tstop,
                };

                target_cache.insert(res);
            };

            let handle_queries = |i: &IntervalNode<PafMetadata, u32>| {
                if i.metadata.name != query_name {
                    return;
                }

                let res = QueryResult {
                    line: i.metadata.line_num,

                    sequence_start: i.first,
                    sequence_stop: i.last,

                    segment_qstart: qstart,
                    segment_qstop: qstop,
                    segment_tstart: tstart,
                    segment_tstop: tstop,
                };

                query_cache.insert(res);
            };

            target_index.query(h_start, h_stop, handle_targets);
            query_index.query(v_start, v_stop, handle_queries);

            target_cache
                .intersection(&query_cache)
                .for_each(|matches: &QueryResult| {
                    match_matches.insert(*matches);
                    // println!("match\t{}\t{}\t{}\t{}\t{}\t{}", target_name, query_name, v_start, v_stop, h_start, h_stop);
                });

            target_cache.intersection(&query_cache).next().is_some()
        };

        let mut traceback_lambda =
            |(q_start, q_stop): (i32, i32), (t_start, t_stop): (i32, i32)| {
                let mut query_cache: HashSet<QueryResult> = HashSet::new();
                let mut target_cache: HashSet<QueryResult> = HashSet::new();

                let handle_targets = |i: &IntervalNode<PafMetadata, u32>| {
                    if i.metadata.name != target_name {
                        return;
                    }

                    let res = QueryResult {
                        line: i.metadata.line_num,

                        sequence_start: i.first,
                        sequence_stop: i.last,

                        segment_qstart: qstart,
                        segment_qstop: qstop,
                        segment_tstart: tstart,
                        segment_tstop: tstop,
                    };

                    target_cache.insert(res);
                };

                let handle_queries = |i: &IntervalNode<PafMetadata, u32>| {
                    if i.metadata.name != query_name {
                        return;
                    }

                    let res = QueryResult {
                        line: i.metadata.line_num,

                        sequence_start: i.first,
                        sequence_stop: i.last,

                        segment_qstart: qstart,
                        segment_qstop: qstop,
                        segment_tstart: tstart,
                        segment_tstop: tstop,
                    };

                    query_cache.insert(res);
                };

                target_index.query(t_start, t_stop, handle_targets);
                query_index.query(q_start, q_stop, handle_queries);

                target_cache
                    .intersection(&query_cache)
                    .for_each(|match_| {
                        // println!("traceback\t{}\t{}\t{}\t{}\t{}\t{}", target_name, query_name, q_start, q_stop, t_start, t_stop);
                        traceback_matches.insert(*match_);
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

        traceback_matches.intersection(&match_matches).for_each(|e| {
            matching_regions.insert(*e);
        })
    }
}

pub fn filter(index: &Index, paf: &paf::PAF, config: &AppConfig) -> Vec<usize> {
    let verbosity = config.verbosity_level;

    let alignment_pairs: HashSet<paf::AlignmentPair> = paf.get_unique_alignments();
    let metadata = paf.get_metadata();

    if verbosity > 1 {
        eprintln!(
            "[wffilter::filter::filter] aligning {} pairs",
            alignment_pairs.len()
        );
    }

    let wflambda_config = wflambda::Config {
        adapt: config.adapt,
        segment_length: config.segment_length as u32, // TODO: remove
        step_size: 500,                               // TODO: remove
        thread_count: config.thread_count,
        verbosity: config.verbosity_level,
    };

    // Progress bar
    let progress_bar = ProgressBar::new(alignment_pairs.len() as u64);
    let template = "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}]  {pos:>7}/{len:7}  ({eta_precise})";
    let progress_style = ProgressStyle::default_bar()
        .template(template)
        .progress_chars("=> ");
    progress_bar.set_style(progress_style);

    // Filter all the alignments
    let all_matching_regions: Vec<HashSet<QueryResult>> = alignment_pairs
        .par_iter()
        .progress_with(progress_bar)
        .map(|alignment_pair: &paf::AlignmentPair| {
            let target_name = &alignment_pair.target_name[..];
            let query_name = &alignment_pair.query_name[..];

            let tlen = metadata.get(target_name).unwrap().length;
            let qlen = metadata.get(query_name).unwrap().length;

            let segments = generate_segments(tlen as usize, qlen as usize, config);

            let mut matching_regions: HashSet<QueryResult> = HashSet::new();

            run_aln(
                &segments,
                index,
                target_name,
                query_name,
                &wflambda_config,
                &mut matching_regions,
            );

            matching_regions
        })
        .collect();

    // Extract the necessary lines
    if all_matching_regions.is_empty() {
        if verbosity > 1 {
            eprintln!(
                "[wffilter::filter::filter] Everything got filtered out. Output PAF will be empty"
            );
        }

        return Vec::new();
    }

    let extract_lines = |query_results: &HashSet<QueryResult>| -> Vec<usize> {
        query_results.iter().map(|query_restult: &QueryResult| query_restult.line as usize).collect()
    };

    let mut lines: Vec<usize> = all_matching_regions
        .iter()
        .map(extract_lines)
        .flatten()
        .collect();

    lines.sort();
    lines.dedup();

    lines
}
