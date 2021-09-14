use crate::filter::types;
use crate::io;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct MashMapLine {
    pub query: String,
    pub query_length: u32,
    pub query_start: u32,
    pub query_stop: u32,

    pub strand: types::Strand, // Relative strand: "+" or "-"

    pub target: String,
    pub target_length: u32,
    pub target_start: u32,
    pub target_stop: u32,
}

impl MashMapLine {
    pub fn from_lines(lines: Vec<String>) -> Vec<MashMapLine> {
        lines
            .iter()
            .map(|line| Self::from_str(&line[..]))
            .collect::<Vec<Self>>()
    }

    pub fn from_str(line: &str) -> Self {
        let it: Vec<&str> = line.split_whitespace().collect();

        //need a more robust way to index into the vector
        Self {
            query: it[0].to_string(),
            query_length: u32::from_str(it[1]).unwrap(),
            query_start: u32::from_str(it[2]).unwrap(),
            query_stop: u32::from_str(it[3]).unwrap(),
            strand: types::Strand::from_char(char::from_str(it[4]).unwrap()),
            target: it[5].to_string(),
            target_length: u32::from_str(it[6]).unwrap(),
            target_start: u32::from_str(it[7]).unwrap(),
            target_stop: u32::from_str(it[8]).unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct MashMapOutput {
    mappings: Vec<MashMapLine>,
}

impl MashMapOutput {
    pub fn from_file(file_name: &str) -> Self {
        let lines: Vec<String> = io::read_file(&file_name[..]);

        let mappings: Vec<MashMapLine> = MashMapLine::from_lines(lines);

        Self { mappings }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_MASHMAP_STRING: &str = "\
    qry1\t11068\t0\t11067\t+\ttgt1\t11068\t0\t11048\t99.9938\
    ";

    static TEST_MASHMAP_FILE: &str = "\
    qry2\t13403\t0\t13402\t+\ttgt2\t13403\t0\t13389\t99.4385\
    \n\
    qry3\t15600\t0\t15599\t-\ttgt3\t15600\t0\t15589\t99.577\
    ";

    #[test]
    fn test_parse_single_alignment() {
        let aln = MashMapLine::from_str(TEST_MASHMAP_STRING);
        let aln2 = MashMapLine {
            query: String::from("qry1"),
            query_length: 11068,
            query_start: 0,
            query_stop: 11067,

            strand: types::Strand::Forward,

            target: String::from("tgt1"),
            target_length: 11068,
            target_start: 0,
            target_stop: 11048,
        };

        assert_eq!(aln, aln2);
    }

    #[test]
    fn test_parse_file() {
        let lines: Vec<String> = TEST_MASHMAP_FILE.lines().map(|x| x.to_string()).collect();
        let file1 = MashMapLine::from_lines(lines);

        let aln2 = MashMapLine {
            query: String::from("qry2"),
            query_length: 13403,
            query_start: 0,
            query_stop: 13402,

            strand: types::Strand::Forward,

            target: String::from("tgt2"),
            target_length: 13403,
            target_start: 0,
            target_stop: 13389,
        };
        let aln3 = MashMapLine {
            query: String::from("qry3"),
            query_length: 15600,
            query_start: 0,
            query_stop: 15599,

            strand: types::Strand::Reverse,

            target: String::from("tgt3"),
            target_length: 15600,
            target_start: 0,
            target_stop: 15589,
        };
        let file2 = vec![aln2, aln3];

        assert_eq!(file1, file2);
    }
}
