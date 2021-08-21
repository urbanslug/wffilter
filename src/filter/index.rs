use coitrees;
use std::convert::TryFrom;
use std::str::FromStr;

use super::types;
use crate::paf;

// TODO: account for strand & stop
fn compute_match_intervals(
    seq_type: types::SequenceType,
    _strand: types::Strand,
    start: u32,
    _stop: u32,
    cigar: &str,
    name: &str,
    line_num: usize,
) -> Vec<types::Interval> {
    let mut intervals: Vec<types::Interval> = Vec::new();
    let mut buffer = String::new();
    let mut cursor = start;

    cigar.chars().for_each(|c: char| {
        match c {
            'M' | '=' => {
                // TODO: consider the ambiguity of M being match/mismatch
                let m: u32 = u32::from_str(&buffer[..]).unwrap();
                intervals.push(types::Interval(cursor, cursor + m, line_num, String::from(name)));
                cursor += m;
                buffer.clear();
            }
            'X' => {
                let x: u32 = u32::from_str(&buffer[..]).unwrap();
                cursor += x;
                buffer.clear();
            }
            'I' => {
                let i: u32 = u32::from_str(&buffer[..]).unwrap();
                if seq_type == types::SequenceType::Target {
                    cursor -= i
                } else {
                    cursor += i
                };
                buffer.clear();
            }
            'D' => {
                let d: u32 = u32::from_str(&buffer[..]).unwrap();
                if seq_type == types::SequenceType::Target {
                    cursor += d
                } else {
                    cursor -= d
                };
                buffer.clear();
            }
            _ => {
                // At this point we expect the char to be a base 10 digit i.e '0', '1', ..., '9'
                match c {
                    b if b.is_digit(10) => buffer.push(b),
                    b if b.is_ascii_alphabetic() => panic!(
                        "[wffilter::filter::index::coompute_match_intervals] Unexpected char {} in CIGAR string",
                        b
                    ),
                    _ => panic!(
                        "[wffilter::filter::index::coompute_match_intervals] Unknown char {} in CIGAR string",
                        c
                    ),
                }
            }
        }
    });

    intervals
}

pub fn index_paf_matches(p: &paf::PAF) -> types::Index {
    let alignments: &Vec<paf::PafAlignment> = p.get_alignments();
    let mut query_intervals: Vec<types::Interval> = Vec::new();
    let mut target_intervals: Vec<types::Interval> = Vec::new();

    alignments
        .iter()
        .enumerate()
        .for_each(|(line_num, a): (usize, &paf::PafAlignment)| {
            let mut t = compute_match_intervals(
                types::SequenceType::Target,
                a.strand,
                a.target_start,
                a.target_end,
                &a.cigar[..],
                &a.target[..],
                line_num,
            );
            let mut q = compute_match_intervals(
                types::SequenceType::Query,
                a.strand,
                a.query_start,
                a.query_end,
                &a.cigar[..],
                &a.query[..],
                line_num,
            );

            query_intervals.append(&mut q);
            target_intervals.append(&mut t);
        });

    let gen_coitree =
        |intervals: Vec<types::Interval>| -> coitrees::COITree<types::PafMetadata, u32> {
            // Generate coitrees::IntervalNodes
            let interval_nodes: Vec<coitrees::IntervalNode<types::PafMetadata, u32>> =
                intervals
                    .iter()
                    .map(|types::Interval(start, stop, line_num, name): &types::Interval| {
                        let start = i32::try_from(*start).expect("[wffilter::filter::index::index_paf] Could not convert start u32 to i32");
                        let end = i32::try_from(*stop).expect("[wffilter::filter::index::index_paf] Could not convert end u32 to i32");
                        let metadata = types::PafMetadata{line_num: *line_num as u32, name: name.clone()};

                        coitrees::IntervalNode::<types::PafMetadata, u32>::new(start, end, metadata)
                    })
                    .collect();

            coitrees::COITree::new(interval_nodes)
        };

    types::Index {
        target_index: gen_coitree(target_intervals),
        query_index: gen_coitree(query_intervals),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod compute_intervals {
        use super::*;

        static TEST_CIGAR_1: &str = "15M1I158M1I24M1I169M1I1147M1I24M1I851M1I13M1I3900M1D25M1I874M4I10847M3D4400M1I1494M1D4041M1I8577M14I1340M2D21138M2I7776M6D3563M2I83120M10D5541M2D27729M1I2M13I49698M1I5030M2I17541M1D22531M1I187M1D458M1D80M1I75M1I266M1I48M1I269M1I460M1D240M";
        static TEST_CIGAR_2: &str = "1I11=1X33=3X5=2X2=1X10=1X7=8I8D5=1X16=1X3=1D9=1X30=1X4=1X9=1X7=1X19=2D4=1X21=1X7=2X17=1X1=2X7=1X2=1X3=2X16=1X9=1X3=1X20=1X2=1X20=1X1=1I4=1X1=1X2=1X13=1D9=1X11=1X4=1X11=1X1=1X32=1X6=3X7=1X1I4=1X5=1X3=1X10=2D23=1X38=1X6=1X6=1X23=1X12=1815D5=141I7=334D6=135I5=115D4=1X1=1X6=61I8=311D5=510D6=878D8=1X233D4=288I4=1X7=52D2=1X5=35D8=208D5=251D6=171I8=161D4=132I6=180D7=92D5=53D4=425I7=44D9=47I5=170I7=788D6=158I8=25D6=159D8=1X2=128D2=1X3=147I7=552D7=463I7=28I6=369I5=30D7=538I9=110I5=36I4=109D12=3I2=48D5=174I9=136D7=172D6=149D2=1X6=367D7=63D4=106I8=212D6=243I4=125D6=121I5=1X5=181D8=134I5=55I8=466D3=603I7=122D6=699D10=130D5=365I5=225D8=35I3=154D10=5I4=58I5=485I8=196D1=1X3=1X5=264I10=135D3=133I11=204D5=161I4=1X5=222I5=251D6=39D6=335I4=6D6=76I3=1X5=490I3=1X10=879D12=7D7=1X2=1X6=2I2=1X13=1X3=2X11=1X4=1X17=1X1=1I29=1X6=1X4=1X9=1X8=1X16=2X4=1X3=1X10=1X3=1X1=1X30=1X2=2D20=1X27=2X4=1X5=1D27=1X10=1X12=1X34=1X19=1X22=1X3=1X17=1X5=1X3=1X6=2X4=1X20=1X46=1X14=1X24=1X7=1X42=1X6=1X19=2X4=1X19=1X16=1X21=1X7=1X19=1X5=1X3=1X3=1X11=1D11=1X2=1X14=1X6=1X10=1X10=1X15=1X4=1X33=1X12=3X11=2X1=1X2=1X13=1X2=1X1=1X12=1X26=3D3=3I6=1X12=1X8=1X4=2X3=1X2=1X12=2X2=1X5=2X10=1X5=3X1=1X10=1X17=1X10=1X47=4D9=1X21=1X34=1X16=1X19=1X3=1X7=1X23=1X11=1X3=1X7=1X38=1X6=1X2=1X4=1X13=1D25=3D2=1X6=1X29=2X1=1X5=9I8=1X19=1X5=1X1=2X2=1X9=1X28=1I9=1X16=1X3=1X2=1X5=1X14=1X3=2X6=2X5=1X18=1X7=1X12=1X26=2X16=1X11=1X2=1X5=1X2=1X13=1X16=6I4=1X6=6D4=5I11=1X11=1X1=1X2=2I5=1X30=1X21=5I2=6D4=1X23=1X20=1X14=1X43=1X24=3D5=1X5=6I2=6D16=1I21=1X3=1X1=1X3=1X3=1X3=1X19=1D1=2X16=1X10=1X6=1X45=1X30=1X16=1X12=1X42=1X4=1X68=1X6=2X24=1X3=1D11=13I";

        #[test]
        fn test_compute_match_intervals_tiny() {
            // Forward
            let intervals_computed: Vec<types::Interval> = compute_match_intervals(
                types::SequenceType::Query,
                types::Strand::Forward,
                0,
                330243,
                "330243M",
                "no_name",
                0,
            );
            let intervals: Vec<types::Interval> =
                vec![types::Interval(0, 330243, 0, String::from("no_name"))];
            assert_eq!(intervals, intervals_computed);
        }

        #[ignore]
        #[test]
        fn test_compute_match_intervals_long_query() {
            let intervals_computed: Vec<types::Interval> = compute_match_intervals(
                types::SequenceType::Query,
                types::Strand::Forward,
                41052,
                324759,
                TEST_CIGAR_1,
                "test_cigar_1",
                0,
            );
            let intervals: Vec<types::Interval> = vec![];
            assert_eq!(intervals, intervals_computed);
        }

        #[ignore]
        #[test]
        fn test_compute_match_intervals_long_target() {
            let intervals_computed: Vec<types::Interval> = compute_match_intervals(
                types::SequenceType::Target,
                types::Strand::Forward,
                0,
                283680,
                TEST_CIGAR_1,
                "test_cigar_1",
                0,
            );
            let intervals: Vec<types::Interval> = vec![];
            assert_eq!(intervals, intervals_computed);
        }

        // TODO: is this cigar even valid?
        #[ignore]
        #[test]
        fn test_failing() {
            let intervals_computed: Vec<types::Interval> = compute_match_intervals(
                types::SequenceType::Query,
                types::Strand::Forward,
                0,
                11068,
                TEST_CIGAR_2,
                "test_cigar_2",
                0,
            );
            let intervals: Vec<types::Interval> = vec![];
            assert_eq!(intervals, intervals_computed);
        }
    }

    mod index {
        use super::*;

        #[test]
        fn test_index_paf() {
            static TEST_PAF_STRING: &str = "\
            qry\t330243\t0\t330243\t+\ttgt\t330243\t0\t330243\t330243\t330243\t60\tNM:i:0\tms:i:660486\tAS:i:660486\tnn:i:0\ttp:A:P\tcm:i:62290\ts1:i:329202\ts2:i:262341\tde:f:0\trl:i:2730\tcg:Z:330243M\
            \n\
            qry\t329347\t41052\t324759\t+\ttgt\t283680\t0\t283680\t283613\t283736\t0\tNM:i:123\tms:i:566760\tAS:i:566760\tnn:i:0\ttp:A:S\tcm:i:53397\ts1:i:282348\tde:f:0.0003\trl:i:2765\tcg:Z:15M1I158M1I24M1I169M1I1147M1I24M1I851M1I13M1I3900M1D25M1I874M4I10847M3D4400M1I1494M1D4041M1I8577M14I1340M2D21138M2I7776M6D3563M2I83120M10D5541M2D27729M1I2M13I49698M1I5030M2I17541M1D22531M1I187M1D458M1D80M1I75M1I266M1I48M1I269M1I460M1D240M
";
            let alignments: paf::PAF = paf::PAF::from_str(TEST_PAF_STRING);
            let index = index_paf_matches(&alignments);
            let query_index = index.query_index;
            let target_index = index.target_index;

            // should apply to all of them
            assert_eq!(38, query_index.query_count(0, 330_243));
            // the first match in the second alignment plus the first alignment which covers everything
            assert_eq!(2, query_index.query_count(41_052, 41_067));
            assert_eq!(query_index.len(), target_index.len());
        }
    }
}
