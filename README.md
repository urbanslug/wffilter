# wffilter

Filter over local alignments with a global alignment.

## Compile

#### Fetch the source
```
git@github.com:urbanslug/wffilter.git
cd wffilter
```
#### cargo
Because this tool is written in Rust, the easiest way to compile it is using
[cargo](https://doc.rust-lang.org/cargo/index.html).
Find the [cargo installation instructions here](https://doc.rust-lang.org/cargo/getting-started/installation.html).

To install into the `target/` dir.
```
cargo build --release
./target/release/wffilter -h
```

To install into the cargo binary path and your $PATH (assuming a standard rust setup).
```
cargo install --path .
wffilter -h
```

## Usage

```
USAGE:
    wffilter [FLAGS] [OPTIONS] <input_paf>

FLAGS:
    -a, --adapt      To apply adaptive wavefront alignment [default: false]
    -h, --help       Prints help information
    -v               Sets the level of verbosity [default: 0]
    -V, --version    Prints version information

OPTIONS:
    -s, --segment-length <segment_length>    Segment length for aligning [default: 10]
    -t, --thread-count <thread_count>        Number of threads to use [default: 8]

ARGS:
    <input_paf>    Path to input PAF file
```

Example
```
wffilter -vv -a -s 100 x.paf > x.filtered.paf 
```

## How it works

### Match index

Reads a PAF file of local alignments from
[minimap2](https://github.com/lh3/minimap2) or
[lastz](https://github.com/lastz/lastz) and creates a cache from the match
regions using [coitrees](https://docs.rs/coitrees/0.2.1/coitrees/index.html).

### Global alignment
This match index is then used by WFA to guide a global alignment through
[wflambda-rs](https://github.com/urbanslug/wflambda-rs) in regions the size of
the argument `--segment-size/-s`.
This guidance involves querying the index to find out where matches are to
fulfill the requirements of the match and traceback lambdas.

## Known issues
I've currently disabled the effect of passing scoring penalties because it
affects performance.



### Citation

**Santiago Marco-Sola, Juan Carlos Moure, Miquel Moreto, Antonio Espinosa**. ["Fast gap-affine pairwise alignment using the wavefront algorithm."](https://doi.org/10.1093/bioinformatics/btaa777) Bioinformatics, 2020.
