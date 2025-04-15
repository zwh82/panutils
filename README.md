# panutils
Some utils for pangenome construction

## Installation
```
# The c_ffi feature need clang. You can install it with conda.
cargo install --path .
```

## Subcommand

```
Usage: panutils <COMMAND>

Commands:
  fastixe  
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### fastixe 

This subcommand is like [fastix](https://github.com/ekg/fastix), which can add prefixes to FASTA headers. It support pangenomic applications, following the [PanSN](https://github.com/pangenome/PanSN-spec) hierarchical naming specification. The difference is to add more functions. 

My idea for rewriting the code in Rust is that [PGGB](https://github.com/pangenome/pggb) requires multiple genomes, each following the `PanSN` naming convention, to be merged into a single file, optionally compressed using `bgzip`. It also needs `samtools faidx` to build an index for the compressed genome file. I hope to implement these features more efficiently using Rust, rather than relying on a pipeline of shell commands involving `fastix`, `bgzip`, and `samtools faidx`. This would reduce dependency on conda-based tools. At the same time, it's also a rust practice project for me as a beginner.

#### Usage

```
Usage: panutils fastixe [OPTIONS]

Options:
  -t, --threads <THREADS>  Number of threads [default: 1]. [default: 1]
      --trace              Trace output (caution: very verbose).
      --debug              Debug output.
  -h, --help               Print help

INPUT FILE:
  -a, --stdin <INPUT_STDIN>             Input stream, use '-' for stdin
  -i, --input-genome <INPUT_GENOME>     Input genome.
  -s, --input-files <INPUT_FILES>...    Multiple input files.
  -l, --input-genome-list <INPUT_LIST>  Input genome list.
  -d, --input-dir <INPUT_DIRECTORY>     Input directory containing FASTA files.

OUTPUT:
  -o, --out-dir <OUT_DIRECTORY>  Output directory. [default: genomes]

SEPARATE OUTPUT:
      --stdout  Stdout.
  -g, --gz      Gzip output.

RENAME:
  -p, --prefix <PREFIX>  Prefix to add to headers.
  -r, --regex <REG>      File name regex [default: [^_]+_[^_]+]

Sequence:
  -u, --up  All bases are converted to uppercase letters.

MERGE OUTPUT:
  -e, --output-file-name <MERGE_OUTPUT_FILE_PATH>  Merge output file path. [default: merged.fa]
  -m, --merge                                      Merge output.
  -b, --bgz                                        Merge bgzip output.

INDEX:
  -f, --faidx  Build the index for fasta's bgzip file, just like samtools faidx.

COMPRESSION LEVEL:
      --level <COMPRESSION_LEVEL>  Compression (0-9).
```

#### Examples
```
# rename with default regex
panutils fastixe -i tests/GCF_002012065.1_ASM201206v1_genomic.fna --up 

# rename with specified prefix
panutils fastixe -i tests/GCF_002012065.1_ASM201206v1_genomic.fna --up -p GCF_002012065.1#0#

# multiple files input 
panutils fastixe -s tests/GCF_002012065.1_ASM201206v1_genomic.fna tests/GCF_006400955.1_ASM640095v1_genomic.fna --up
panutils fastixe -l tests/test_genome_list.txt --up
panutils fastixe -d tests/ --up

# gzip output
panutils fastixe -i tests/GCF_002012065.1_ASM201206v1_genomic.fna --up -g

# merge and bgzip output and faidx
panutils fastixe -d tests/ -m -b -f -e test_merged.fa --up
```


