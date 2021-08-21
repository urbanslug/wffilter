use chrono::{DateTime, Local};

#[derive(Copy, Clone, Debug)]
pub struct Penalties {
    pub mismatch: u64,
    pub matches: u64,
    pub gap_open: u64,
    pub gap_extend: u64,
}

#[derive(Debug)]
pub struct AppConfig {
    pub input_paf: String,

    pub segment_length: usize,
    pub step: bool, // Use overlapping segments or not? Set to true everywhere for now

    pub thread_count: usize,
    pub penalties: Penalties,
    pub adapt: bool,
    pub verbosity_level: u8,
    pub start_time: DateTime<Local>,
}

impl AppConfig {
    #[allow(dead_code)] // TODO: exists for testing purposes
    pub fn new(
        paf_filepath: &str,
        segment_length: usize,
        thread_count: usize,
        penalties: Option<Penalties>,
        adapt: bool,
        verbosity_level: u8,
    ) -> Self {
        let penalties = match penalties {
            Some(p) => p,
            _ => Penalties {
                // default penalties
                mismatch: 1,
                matches: 0,
                gap_open: 1,
                gap_extend: 1,
            },
        };

        AppConfig {
            verbosity_level,
            input_paf: String::from(paf_filepath),
            segment_length,
            step: true,
            thread_count,
            penalties,
            adapt,
            start_time: Local::now(),
        }
    }
}
