mod cli;
mod filter;
mod io;
mod paf;
mod types;

use std::time::Instant;

fn main() {
    let total_time = Instant::now();

    // ------------
    //    CLI
    // ------------

    // Parse CLI args
    let config: types::AppConfig = cli::start();
    let paf_file_path: &str = &config.input_paf[..];
    let verbosity = config.verbosity_level;

    // TODO: remove
    if verbosity > 1 {
        eprintln!("{:#?}", config)
    }

    // ------------
    //     PAF
    // ------------

    // Parse the PAF input file
    let now = Instant::now();
    if verbosity > 0 {
        eprintln!("[wffilter::main] parsing PAF: {}", paf_file_path);
    }

    let paf = paf::PAF::from_file(paf_file_path);
    if verbosity > 1 {
        eprintln!(
            "[wffilter::main] done parsing PAF. Time taken {} seconds",
            now.elapsed().as_millis() as f64 / 1000.0
        )
    }

    // ------------
    //     Index
    // ------------

    let now = Instant::now();
    if verbosity > 0 {
        eprintln!("[wffilter::main] indexing");
    }

    // Generate the index
    let index: filter::types::Index = filter::index::index_paf_matches(&paf);

    if verbosity > 1 {
        eprintln!(
            "[wffilter::main] done indexing. Time taken {} seconds",
            now.elapsed().as_millis() as f64 / 1000.0
        )
    }

    // ------------
    //     Filter
    // ------------
    let now = Instant::now();
    if verbosity > 0 {
        eprintln!("[wffilter::main] filtering");
    }

    filter::filter::filter(&index, &paf, &config);

    if verbosity > 1 {
        eprintln!(
            "[wffilter::main] done filtering. Time taken {} seconds",
            now.elapsed().as_millis() as f64 / 1000.0
        )
    }

    // generate segments
    // config
    // let tlen: usize = 60 as usize;
    // let qlen: usize = 60 as usize;
    // let segment_length: usize = 10;
    // let segments = filter::filter::generate_segments(tlen, qlen, segment_length);
    // println!("The segments are:\n {:?}\n\n", segments);
    // eprintln!("The segments are");
    // utils::pretty_print_vec(&segments, 3);
    // eprintln!();

    // build Index
    // let index = filter::filter::build_index();
    // let (text_lines, query_lines) = filter::filter::run_align(&segments, &index);

    // query the index
    // are there matches in this segment?
    // let _overlap: Vec<&filter::types::QueryResult> = text_lines.intersection(&query_lines).collect();
    // eprintln!("Number of results: {}", overlap.len());
    // eprintln!();

    // for x in overlap.iter() {
    //   println!("{}", x);
    //}

    if verbosity > 1 {
        eprintln!(
            "[wffilter::main] Done. Total time taken {} seconds",
            total_time.elapsed().as_millis() as f64 / 1000.0
        )
    }
}
