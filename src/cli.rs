use clap::{App, Arg};
use std::env;

use crate::types;

// Env vars
const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

pub fn start() -> types::AppConfig {
    let matches = App::new(NAME)
        .version(VERSION)
        .author(AUTHORS)
        .about(DESCRIPTION)
        .arg(
            Arg::with_name("input_paf")
                .required(true)
                .takes_value(true)
                .help("Path to input PAF file"),
        )
        .arg(
            Arg::with_name("segment_length")
                .short("s")
                .long("segment-length")
                .multiple(false)
                .default_value("256")
                .help("Segment length for aligning"),
        )
        .arg(
            Arg::with_name("thread_count")
                .short("t")
                .long("thread-count")
                .default_value("8")
                .takes_value(true)
                .help("Number of threads to use"),
        )
        .arg(
            Arg::with_name("adapt")
                .short("a")
                .long("adapt")
                .multiple(false)
                .help("To apply adaptive wavefront alignment [default: false]"),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity [default: 0]"),
        )
        .get_matches();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    let paf_file_path: &str = matches.value_of("input_paf").unwrap();
    let segment_length: usize = matches
        .value_of("segment_length")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let thread_count: usize = matches
        .value_of("thread_count")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let adapt: bool = matches.is_present("adapt");
    let verbosity_level: u8 = matches.occurrences_of("v") as u8;

    types::AppConfig::new(
        paf_file_path,
        segment_length,
        thread_count,
        None, // TODO: implement penalties
        adapt,
        verbosity_level,
    )
}
