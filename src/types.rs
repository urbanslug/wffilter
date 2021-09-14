use chrono::{DateTime, Local};

#[derive(Copy, Clone, Debug)]
pub struct Penalties {
    pub mismatch: u8,
    pub matches: u8,
    pub gap_open: u8,
    pub gap_extend: u8,
}

#[derive(Debug)]
pub struct AppConfig {
    pub input_paf: String,

    pub mashmap_filepath: Option<String>,

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
        mashmap_filepath_: Option<&str>,
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

        let mashmap_filepath: Option<String> = match mashmap_filepath_ {
            Some(fp) => Some(String::from(fp)),
            _ => None,
        };

        AppConfig {
            verbosity_level,
            input_paf: String::from(paf_filepath),
            mashmap_filepath,
            segment_length,
            step: true,
            thread_count,
            penalties,
            adapt,
            start_time: Local::now(),
        }
    }
}
