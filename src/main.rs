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

    let filtered_lines = filter::filter::filter(&index, &paf, &config);

    if verbosity > 1 {
        eprintln!(
            "[wffilter::main] done filtering. Time taken {} seconds",
            now.elapsed().as_millis() as f64 / 1000.0
        )
    }

    // ------------
    //     Copy over lines
    // ------------
    let now = Instant::now();
    if verbosity > 0 {
        eprintln!("[wffilter::main] copying filtered lines");
    }

    io::copy_filtered(&config.input_paf[..], &filtered_lines);

    if verbosity > 1 {
        eprintln!(
            "[wffilter::main] done copying. Time taken {} seconds",
            now.elapsed().as_millis() as f64 / 1000.0
        )
    }

    if verbosity > 1 {
        eprintln!(
            "[wffilter::main] Done. Total time taken {} seconds",
            total_time.elapsed().as_millis() as f64 / 1000.0
        )
    }
}
