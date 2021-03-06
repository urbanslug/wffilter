mod cli;
mod filter;
mod io;
mod mashmap;
mod paf;
mod types;

use std::time::Instant;

fn main() {
    let total_time = Instant::now();

    // -----------------
    //    CLI & Config
    // -----------------

    // Parse CLI args
    let config: types::AppConfig = cli::start();
    let paf_file_path: &str = &config.input_paf[..];
    let verbosity = config.verbosity_level;

    // Initialization of the global thread pool happens exactly once.
    // Once started, the configuration cannot be changed.
    // Therefore, if you call build_global a second time, it will return an error.
    rayon::ThreadPoolBuilder::new()
        .num_threads(config.thread_count)
        .build_global()
        .unwrap();

    // TODO: remove
    if verbosity > 1 {
        eprintln!("{:#?}", config)
    }

    // ------------
    //     Mashmap
    // ------------

    let mut mashmap_mappings: Option<mashmap::MashMapOutput> = None;
    if config.mashmap_filepath.is_some() {
        let foo: Option<&String> = config.mashmap_filepath.as_ref();
        let mashmap_file_path: &str = &foo.unwrap()[..];
        let now = Instant::now();
        if verbosity > 0 {
            eprintln!(
                "[wffilter::main] parsing mashmap output: {}",
                mashmap_file_path
            );
        }

        mashmap_mappings = Some(mashmap::MashMapOutput::from_file(mashmap_file_path));
        if verbosity > 1 {
            eprintln!(
                "[wffilter::main] done parsing mashmap output. Time taken {} seconds.",
                now.elapsed().as_millis() as f64 / 1000.0
            )
        }
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
            "[wffilter::main] done parsing PAF. Time taken {} seconds.",
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
            "[wffilter::main] done indexing. Time taken {} seconds.",
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

    // todo: pass this to fns
    let mut filtered_lines: Vec<usize>;

    if config.mashmap_filepath.is_some() {
        filtered_lines = filter::filter::filter_mashmap(mashmap_mappings.as_ref(), &index, &paf, &config);
    } else {
        filtered_lines = filter::filter::filter(&index, &paf, &config);
    }

    if verbosity > 1 {
        eprintln!(
            "[wffilter::main] done filtering. Time taken {} seconds.",
            now.elapsed().as_millis() as f64 / 1000.0
        )
    }

    // --------------------------
    //     Generate filtered PAF
    // --------------------------

    let now = Instant::now();
    if verbosity > 0 {
        eprintln!("[wffilter::main] copying filtered lines");
    }

    io::copy_filtered(paf_file_path, &filtered_lines);

    if verbosity > 1 {
        eprintln!(
            "[wffilter::main] done copying. Time taken {} seconds.",
            now.elapsed().as_millis() as f64 / 1000.0
        )
    }

    if verbosity > 1 {
        eprintln!(
            "[wffilter::main] all done. Total time taken {} seconds.",
            total_time.elapsed().as_millis() as f64 / 1000.0
        )
    }
}
