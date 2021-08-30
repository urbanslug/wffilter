# wffilter

Filter over local alignments with a global alignment.

## Match index

Reads a PAF file of local alignments from
[minimap2](https://github.com/lh3/minimap2) or
[lastz](https://github.com/lastz/lastz) and creates a cache from the match
regions using [coitrees](https://docs.rs/coitrees/0.2.1/coitrees/index.html).

## Global alignment
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
