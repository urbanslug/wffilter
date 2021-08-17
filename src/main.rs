mod filter;
mod wflambda;

fn main() {
    // config
    let tlen: usize = 60 as usize;
    let qlen: usize = 60 as usize;
    let segment_length: usize = 5;

    // generate segments
    let segments = filter::generate_segments(tlen, qlen, segment_length);
    // println!("The segments are:\n {:?}\n\n", segments);
    // eprintln!("The segments are");
    // utils::pretty_print_vec(&segments, 3);
    // eprintln!();

    // build Index
    let index = filter::build_index();
    let (text_lines, query_lines) = filter::run_align(&segments, &index);

    // query the index
    // are there matches in this segment?
    let _overlap: Vec<&filter::QueryResult> = text_lines.intersection(&query_lines).collect();
    // eprintln!("Number of results: {}", overlap.len());
    // eprintln!();

    // for x in overlap.iter() {
    //   println!("{}", x);
    //}
}
