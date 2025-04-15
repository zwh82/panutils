
use clap::{Parser, Args, Subcommand};
use std::path::PathBuf;


#[derive(Parser)]
#[clap(author, version, arg_required_else_help = true, about = "panutils")]
pub struct Cli {
    #[clap(subcommand,)]
    pub mode: Mode,
}


#[derive(Subcommand)]
pub enum Mode {
    #[clap(arg_required_else_help = true, display_order = 1)]
    Fastixe(FastixeArgs),
}

#[derive(Args, Default, Debug)]
pub struct FastixeArgs {

    #[clap(short = 'a', long = "stdin", help_heading = "INPUT FILE", help = "Input stream, use '-' for stdin")]
    pub input_stdin: Option<String>,

    #[clap(short = 'i', long = "input-genome", help_heading = "INPUT FILE", help = "Input genome.")]
    pub input_genome: Option<PathBuf>,

    #[clap(short = 's', long = "input-files", help_heading = "INPUT FILE", num_args = 1.., help = "Multiple input files.")]
    pub input_files: Option<Vec<PathBuf>>,

    #[clap(short = 'l', long = "input-genome-list", help_heading = "INPUT FILE", help = "Input genome list.")]
    pub input_list: Option<PathBuf>,

    #[clap(short = 'd', long = "input-dir", help_heading = "INPUT FILE", help = "Input directory containing FASTA files.")]
    pub input_directory: Option<PathBuf>,

    #[clap(short = 'o', long = "out-dir", default_value = "genomes", help_heading = "OUTPUT", help = "Output directory.")]
    pub out_directory: PathBuf,

    #[clap(long = "stdout", help_heading = "SEPARATE OUTPUT", help = "Stdout.")]
    pub is_stdout: bool,

    #[clap(short = 'p', long = "prefix", help_heading = "RENAME", help = "Prefix to add to headers.")]
    pub prefix: Option<String>,

    #[clap(short = 'r', long = "regex", default_value_t = String::from("[^_]+_[^_]+"), help_heading = "RENAME", help = "File name regex")]
    pub reg: String,

    #[clap(short, long="up", help_heading = "Sequence", help = "All bases are converted to uppercase letters.")]
    pub uppercase: bool,

    #[clap(short, long="gz", help_heading = "SEPARATE OUTPUT", help = "Gzip output.")]
    pub gzip_output: bool,

    #[clap(short = 'e', long="output-file-name", default_value_t = String::from("merged.fa"), help_heading = "MERGE OUTPUT", help = "Merge output file path.")]
    pub merge_output_file_path: String,

    #[clap(short, long="merge", help_heading = "MERGE OUTPUT", help = "Merge output.")]
    pub merge_output: bool,

    #[clap(short = 'b', long="bgz", help_heading = "MERGE OUTPUT", help = "Merge bgzip output.")]
    pub merge_bgzip_output: bool,    

    #[clap(short, long="faidx", help_heading = "INDEX", help = "Build the index for fasta's bgzip file, just like samtools faidx.")]
    pub faidx: bool,

    #[clap(long = "level", help_heading = "COMPRESSION LEVEL", help = "Compression (0-9).")]
    pub compression_level: Option<u32>,

    #[clap(short = 't', long = "threads", default_value_t = 1, help = "Number of threads [default: 1].")]
    pub threads: usize,

    #[clap(long="trace", help = "Trace output (caution: very verbose).")]
    pub trace: bool,
    #[clap(long="debug", help = "Debug output.")]
    pub debug: bool,
}