/*
PAF: a Pairwise mApping Format
Spec https://github.com/lh3/miniasm/blob/master/PAF.md

PAF is a text format describing the approximate mapping positions between two
set of sequences. PAF is TAB-delimited with each line consisting of the
following predefined fields:

|Col|Type   |Description                               |
|--:|:-----:|:-----------------------------------------|
|1  |string |Query sequence name                       |
|2  |int    |Query sequence length                     |
|3  |int    |Query start (0-based; BED-like; closed)   |
|4  |int    |Query end (0-based; BED-like; open)       |
|5  |char   |Relative strand: "+" or "-"               |
|6  |string |Target sequence name                      |
|7  |int    |Target sequence length                    |
|8  |int    |Target start on original strand (0-based) |
|9  |int    |Target end on original strand (0-based)   |
|10 |int    |Number of residue matches                 |
|11 |int    |Alignment block length                    |
|12 |int    |Mapping quality (0-255; 255 for missing)  |

If PAF is generated from an alignment, column 10 equals the number of sequence
matches, and column 11 equals the total number of sequence matches, mismatches,
insertions and deletions in the alignment. If alignment is not available,
column 10 and 11 are still required but may be highly inaccurate.

A PAF file may optionally contain SAM-like typed key-value pairs at the end of
each line.
*/

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::str;
use std::str::FromStr;

use crate::filter::types;
use crate::io;

// A struct over a single line of a PAF file (a single alignment)
#[derive(PartialEq, Debug)]
pub struct PafAlignment {
    pub query: String,         // Query sequence name
    query_length: u32,         // Query sequence length
    pub query_start: u32,      // Query start (0-based; BED-like; closed)
    pub query_end: u32,        // Query end (0-based; BED-like; open)
    pub strand: types::Strand, // Relative strand: "+" or "-"
    pub target: String,        // target sequence name
    target_length: u32,        // Target sequence length
    pub target_start: u32,     // Target start on original strand (0-based)
    pub target_end: u32,       // Target end on original strand (0-based)
    // residue_matches: u32,  // Number of residue matches
    // block_len: u32,        // Alignment block length
    // quality: String,       // Mapping quality (0-255; 255 for missing)
    pub cigar: String, // SAM style CIGAR string TODO: specify CIGAR version
}

// TODO: remove
#[allow(dead_code)]
pub type Alignment = PafAlignment;

impl PafAlignment {
    #[allow(dead_code)] //TODO: Used in testing. Remove?
    pub fn new(
        query: &str,
        query_length: u32,
        query_start: u32,
        query_end: u32,
        strand: types::Strand,
        target: &str,
        target_length: u32,
        target_start: u32,
        target_end: u32,
        cigar: &str,
    ) -> Self {
        PafAlignment {
            query: String::from(query),
            query_length,
            query_start,
            query_end,
            strand,
            target: String::from(target),
            target_length,
            target_start,
            target_end,
            cigar: String::from(cigar),
        }
    }
    pub fn from_lines(lines: Vec<String>) -> Vec<PafAlignment> {
        lines
            .iter()
            .map(|line| PafAlignment::from_str(&line[..]))
            .collect::<Vec<PafAlignment>>()
    }

    pub fn from_str(line: &str) -> Self {
        let it: Vec<&str> = line.split_whitespace().collect();
        let sam_fields: &[&str] = &it[9..];

        // expensive
        // should return an Option<String> in case a field doens't exist
        let extract_field = |pattern: &str| -> String {
            let f: &str = sam_fields
                .iter()
                .find(|s| s.find(pattern) != None) // if the field is found
                .unwrap();

            // drop the cg chars
            String::from(&f[5..])
        };

        let extract_strand = || -> types::Strand {
            if '+' == char::from_str(it[4]).unwrap() {
                types::Strand::Forward
            } else {
                types::Strand::Reverse
            }
        };

        //need a more robust way to index into the vector
        PafAlignment {
            query: it[0].to_string(),
            query_length: u32::from_str(it[1]).unwrap(),
            query_start: u32::from_str(it[2]).unwrap(),
            query_end: u32::from_str(it[3]).unwrap(),
            strand: extract_strand(),
            target: it[5].to_string(),
            target_length: u32::from_str(it[6]).unwrap(),
            target_start: u32::from_str(it[7]).unwrap(),
            target_end: u32::from_str(it[8]).unwrap(),
            cigar: extract_field("cg"),
        }
    }
}

impl fmt::Display for PafAlignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("")
            .field("query start", &self.query_start)
            .field("query end", &self.query_end)
            .field("strand", &self.strand)
            .field("target start", &self.target_start)
            .field("target end", &self.target_end)
            .field("cigar", &self.cigar)
            .finish()
    }
}

// TODO: merge with filter::types::PafMetadata
#[derive(Clone, Copy, Debug)]
pub struct PafMetadata {
    pub length: u32,
}

// TODO: should this be part of PAF?
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct AlignmentPair {
    pub target_name: String,
    pub query_name: String,
}

type PafLookup = HashMap<String, PafMetadata>;

// A struct over the entire PAF file
#[derive(Debug)]
pub struct PAF {
    alignments: Vec<PafAlignment>,
    metadata: PafLookup,
}

impl PAF {
    pub fn from_file(file_name: &str) -> PAF {
        // read PAF file & return a vector of Strings for each line
        let lines: Vec<String> = io::read_file(&file_name[..]);

        // Parse each line into a PafAlignment
        let alignments: Vec<PafAlignment> = PafAlignment::from_lines(lines);

        let metadata = Self::populate_metadata(&alignments);

        PAF {
            alignments,
            metadata,
        }
    }

    // A string of alignment lines seperated by newlines
    // TODO: Used in testing. Remove?
    #[allow(dead_code)]
    pub fn from_str(alignment_strings: &str) -> PAF {
        let alignments: Vec<PafAlignment> = alignment_strings
            .lines()
            .map(|l| PafAlignment::from_str(l))
            .collect();

        let metadata = Self::populate_metadata(&alignments);

        PAF {
            alignments,
            metadata,
        }
    }

    fn populate_metadata(alignments: &Vec<PafAlignment>) -> PafLookup {
        let mut lookup: PafLookup = HashMap::new();
        for alignment in alignments {
            lookup.insert(
                alignment.query.clone(),
                PafMetadata {
                    length: alignment.query_length,
                },
            );
            lookup.insert(
                alignment.target.clone(),
                PafMetadata {
                    length: alignment.target_length,
                },
            );
        }

        lookup
    }

    pub fn get_alignments(&self) -> &Vec<PafAlignment> {
        &self.alignments
    }

    pub fn get_metadata(&self) -> &PafLookup {
        &self.metadata
    }

    // returns pairs (without duplicates) of the alignments
    // what was the query and the target()
    // this helps speed up alignment
    pub fn get_unique_alignments(&self) -> HashSet<AlignmentPair> {
        self.alignments
            .iter()
            .map(|aln: &PafAlignment| AlignmentPair {
                target_name: aln.target.clone(),
                query_name: aln.query.clone(),
            })
            .collect::<HashSet<AlignmentPair>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_PAF_STRING: &str = "qry\t330243\t0\t330243\t+\ttgt\t330243\t0\t330243\t330243\t330243\t60\tNM:i:0\tms:i:660486\tAS:i:660486\tnn:i:0\ttp:A:P\tcm:i:62290\ts1:i:329202\ts2:i:262341\tde:f:0\trl:i:2730\tcg:Z:330243M";

    #[test]
    fn test_parse_alignment() {
        let aln = PafAlignment::from_str(TEST_PAF_STRING);
        let aln2 = PafAlignment::new(
            "qry",
            330243,
            0,
            330243,
            types::Strand::Forward,
            "tgt",
            330243,
            0,
            330243,
            "330243M",
        );

        assert_eq!(aln, aln2);
    }

    #[test]
    fn test_paf_metadata() {
        let paf = PAF::from_str(TEST_PAF_STRING);
        let mut lookup: PafLookup = HashMap::new();
        lookup.insert(String::from("qry"), PafMetadata { length: 330243 });
        lookup.insert(String::from("tgt"), PafMetadata { length: 330243 });

        let metadata = paf.get_metadata();

        assert_eq!(metadata.len(), lookup.len());
    }

    #[test]
    fn test_paf_unique_alignments() {
        let paf = PAF::from_str(TEST_PAF_STRING);
        let mut unique_alignments: HashSet<AlignmentPair> = HashSet::new();

        unique_alignments.insert(AlignmentPair {
            target_name: String::from("tgt"),
            query_name: String::from("qry"),
        });

        assert_eq!(paf.get_unique_alignments(), unique_alignments);
    }
}
