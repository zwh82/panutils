
use std::path::{Path, PathBuf};
use std::fs::{read_dir, File, create_dir_all};
use std::io::{BufReader, BufRead, BufWriter, Write, stdin, stdout};
use flate2::read::MultiGzDecoder;

use rayon::prelude::*;
use crossbeam_channel::unbounded;
use needletail::parse_fastx_file;
use regex::Regex;
// use bgzip::write::BGZFMultiThreadWriter;
use crate::cmdline::*;
use log::*;

#[cfg(not(feature = "c_ffi"))]
mod not_c_ffi_imports {
    pub use flate2::write::GzEncoder;
    pub use flate2::Compression;
}

#[cfg(not(feature = "c_ffi"))]
use not_c_ffi_imports::*;

#[cfg(feature = "c_ffi")]
mod c_ffi_imports {
    pub use rust_htslib::bgzf::{Writer as BGZFWriter, CompressionLevel};
    pub use rust_htslib::tpool::ThreadPool;
    pub use rust_htslib::faidx::build;
    pub use libdeflater::{Compressor, CompressionLvl};
}

#[cfg(feature = "c_ffi")]
use c_ffi_imports::*;



#[cfg(feature = "c_ffi")]
struct GzipDeflaterWriter<W: Write> {
    inner: W,
    compressor: Compressor,
    buffer: Vec<u8>,
}

#[cfg(feature = "c_ffi")]
impl<W: Write> GzipDeflaterWriter<W> {
    pub fn new(inner: W, level: Option<i32>) -> Self {
        let lvl = level
            .and_then(|l| CompressionLvl::new(l as i32).ok())
            .unwrap_or_else(CompressionLvl::default);

        Self {
            inner,
            compressor: Compressor::new(lvl),
            buffer: Vec::new(),
        }
    }
}

#[cfg(feature = "c_ffi")]
impl<W: Write> Write for GzipDeflaterWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let max_size = self.compressor.gzip_compress_bound(buf.len());
        self.buffer.resize(max_size, 0);
        let compressed_size = self
            .compressor
            .gzip_compress(buf, &mut self.buffer)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        self.inner.write_all(&self.buffer[..compressed_size])?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

fn check_args_valid(args: &FastixeArgs) {
    let level: LevelFilter;
    if args.trace {
        level = log::LevelFilter::Trace;
    } else if args.debug {
        level = log::LevelFilter::Debug;
    } else {
        level = log::LevelFilter::Info
    }

    simple_logger::SimpleLogger::new()
        .with_level(level)
        .init()
        .unwrap();

    rayon::ThreadPoolBuilder::new().num_threads(args.threads).build_global().unwrap();

    if args.input_genome.is_none()
        && args.input_list.is_none()
        && args.input_directory.is_none()
        && args.input_files.is_none()
    {
        if args.input_stdin.is_none() {
            error!("No genome found! Exiting.");
            std::process::exit(1); 
        }  
    } else {
        if args.input_stdin.is_some() {
            error!("Input stream option --stdin cannot be shared with other input options.");
            std::process::exit(1);            
        }    
    }

    if args.input_stdin.is_some() && args.prefix.is_none() {
        error!("Input stream option --stdin need provide prefix.");
        std::process::exit(1);         
    }

    // if args.prefix.is_none() {
    //     warn!("No prefix provided; use default regex.");
    // }
}

fn parse_line_file(path: &Path, vec: &mut Vec<String>) -> std::io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        let path: &Path = line.as_ref();
        if path.exists() {
            vec.push(line);
        } else {
            eprintln!("{:?} does not exist!", path);
        }
    }
    Ok(())
}

fn is_fasta<P: AsRef<Path>>(file: P) -> bool {
    let path = file.as_ref();

    if let Some(file_name) = path.to_str() {
        file_name.ends_with(".fa") ||
            file_name.ends_with(".fna") ||
            file_name.ends_with(".fasta") ||
            file_name.ends_with(".fa.gz") ||
            file_name.ends_with(".fna.gz") ||
            file_name.ends_with(".fasta.gz")
    } else {
        eprintln!("{:?} can not convert to utf-8 string", path);
        false
    }

}

fn parse_files(args: &FastixeArgs, input_genomes: &mut Vec<String>) {
    let mut all_files = vec![];

    if let Some(ref input_stdin) = args.input_stdin {
        all_files.push(input_stdin.to_string());
    }

    if let Some(ref input_genome) = args.input_genome {
        if input_genome.exists() {
            all_files.push(input_genome.to_string_lossy().to_string());
        } else {
            eprintln!("{:?} does not exist!", input_genome)
        }
    }

    if let Some(ref input_files) = args.input_files {
        for input_file in input_files {
            if input_file.is_file() && is_fasta(&input_file) {
                all_files.push(input_file.to_string_lossy().to_string());
            }
        }
    }

    if let Some(ref input_list) = args.input_list {
        if input_list.exists() {
            parse_line_file(&input_list, &mut all_files).unwrap();
        }
    }

    if let Some(ref input_directory) = args.input_directory {
        if input_directory.exists() && input_directory.is_dir() {
            for entry in read_dir(&input_directory).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_file() && is_fasta(&path) {
                    all_files.push(path.to_string_lossy().to_string());
                }
            }
        }
    }


    input_genomes.extend(all_files);

}

fn open_reader(file_path: &Path) -> std::io::Result<Box<dyn BufRead>> {

    if file_path == Path::new("-") {
        let stdin = stdin();
        let handle = stdin.lock();
        let reader: Box<dyn BufRead> = Box::new(BufReader::new(handle));
        Ok(reader)
    } else {
        let input_file = File::open(file_path)?;
        let reader: Box<dyn BufRead> = if file_path.extension().map_or(false, |e| e == ".gz") {
            Box::new(BufReader::new(MultiGzDecoder::new(input_file)))
        } else {
            Box::new(BufReader::new(input_file))
        };        
        Ok(reader)
    }
}

fn process_fasta(file_path: &Path, output_file_path: &Path, prefix: &str, uppercase: bool, gzip_output: bool, compression_level: Option<u32>, is_stdout: bool) -> std::io::Result<()> {
    let reader = open_reader(file_path)?;
    // let compression = compression_level
    //     .map(Compression::new)
    //     .unwrap_or(Compression::default());
    // let writer: Box<dyn Write> = if is_stdout || output_file_path == Path::new("-") {
    //     let stdout = stdout();
    //     if gzip_output {
    //         let handle = stdout.lock();
    //         Box::new(GzEncoder::new(handle, compression))            
    //     } else {
    //         Box::new(stdout.lock())
    //     }
    // } else {
    //     let output_file = File::create(output_file_path)?;
    //     if gzip_output {
    //         Box::new(GzEncoder::new(output_file, compression))
    //     } else {
    //         Box::new(output_file)
    //     }
    // };

    let writer: Box<dyn Write> = if is_stdout || output_file_path == Path::new("-") {
        let handle = stdout().lock();

        if gzip_output {
            #[cfg(not(feature = "c_ffi"))]
            {
                let compression = compression_level.map(Compression::new).unwrap_or(Compression::default());
                Box::new(GzEncoder::new(handle, compression))
            }

            #[cfg(feature = "c_ffi")]
            {
                Box::new(GzipDeflaterWriter::new(handle, compression_level.map(|v| v as i32)))
            }
        } else {
            Box::new(handle)
        }
    } else {
        let output_file = File::create(output_file_path)?;
        if gzip_output {
            #[cfg(not(feature = "c_ffi"))]
            {
                let compression = compression_level.map(Compression::new).unwrap_or(Compression::default());
                Box::new(GzEncoder::new(output_file, compression))
            }

            #[cfg(feature = "c_ffi")]
            {
                Box::new(GzipDeflaterWriter::new(output_file, compression_level.map(|v| v as i32)))
            }
        } else {
            Box::new(output_file)
        }
    };
    let mut writer = BufWriter::new(writer);
    
    for line in reader.lines() {
        let line = line?;
        if line.starts_with(">") {
            if let Some(record_id) = line[1..].split_whitespace().next() {
                writeln!(writer, ">{}{}", prefix, record_id)?;
            } else {
                eprintln!("Missing recored id in file: {:?}", file_path);
            }
        } else {
            if uppercase {
                writeln!(writer, "{}", line.to_ascii_uppercase())?;
            } else {
                writeln!(writer, "{}", line)?;
            }
            
        }
    }
    Ok(())
}



fn make_output_path(input_file_path: &Path, output_dir_path: &Path, gzip_output: bool) -> PathBuf {
    // let input_file_stem = input_file_path.file_stem().unwrap().to_string_lossy();
    // let ext = if gzip_output {"fa.gz"} else {"fa"};
    // output_dir_path.join(format!("{input_file_stem}.{ext}"))
    let input_file_name = input_file_path.file_name().unwrap().to_string_lossy();
    if gzip_output {
        output_dir_path.join(format!("{input_file_name}.gz"))
    } else {
        output_dir_path.join(format!("{input_file_name}"))
    }
}


fn extract_prefix_from_path(file_path: &Path, regex: &str) -> Result<String, std::io::Error> {
    let input_file_name = file_path.file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid file path"))?
        .to_string_lossy();

    let re = Regex::new(regex)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Invalid regex: {}", e)))?;

    match re.captures(&input_file_name) {
        Some(caps) => {
            let matched = caps.get(0).map(|m| m.as_str()).unwrap_or("");
            Ok(format!("{}#0#", matched))
        }
        None => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Filename '{}' doesn't match regex '{}'",
                input_file_name, regex
            ),
        )),
    }
}

fn process_fasta_needle(file_path: &Path, regex: &str, uppercase: bool) -> Result<Vec<String>, std::io::Error> {
    let mut results = vec![];
    
    // let input_file_name = file_path.file_name().unwrap().to_string_lossy();
    // let re = Regex::new(regex).map_err(|e| {
    //     std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Invalid regex: {}", e))
    // })?;
    // let prefix = match re.captures(&input_file_name) {
    //     Some(caps) => {
    //         let matched = caps.get(0).map(|m| m.as_str()).unwrap_or("");
    //         format!("{}#0#", matched)
    //     }
    //     None => {
    //         return Err(std::io::Error::new(
    //             std::io::ErrorKind::InvalidInput,
    //             format!(
    //                 "Filename '{}' doesn't match regex '{}'",
    //                 input_file_name, regex
    //             ),
    //         ));
    //     }
    // };
    let prefix = extract_prefix_from_path(file_path, regex)?;

    if let Ok(mut reader) = parse_fastx_file(file_path) {
        while let Some(record) = reader.next() {
            if let Ok(seqrec) = record {
                if let Some(first_record_id) = std::str::from_utf8(seqrec.id()).unwrap().split_whitespace().next() {
                    if let Ok(seq) = std::str::from_utf8(seqrec.raw_seq()) {
                        let seq_formatted = if uppercase {
                            seq.to_ascii_uppercase()
                        } else {
                            seq.to_string()
                        };
                        results.push(format!(">{}{}\n{}\n", prefix, first_record_id, seq_formatted));
                    } else {
                        eprintln!("Invalid UTF-8 sequence in file: {:?}", file_path);
                    }
                } else {
                    eprintln!("Missing recored id in file: {:?}", file_path);
                }
                
            }
        }
    }
    Ok(results)
}

#[allow(unused_variables)]
fn create_all_fasta_and_merge_writer(output_file_path: &Path, bgzip_output: bool, compression_level: Option<u32>, threads: usize) -> std::io::Result<Box<dyn Write>> {
    if bgzip_output {
        #[cfg(feature = "c_ffi")]
        {
            let compression = compression_level
                .map(|l| CompressionLevel::Level(l as i8))
                .unwrap_or(CompressionLevel::Default);
            let mut writer = BGZFWriter::from_path_with_level(output_file_path, CompressionLevel::Default)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            let tpool = ThreadPool::new(threads as u32)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            writer
                .set_thread_pool(&tpool)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            Ok(Box::new(writer))
        }

        #[cfg(not(feature = "c_ffi"))]
        {
            eprintln!("Error: `--merge_bgzip_output` requires feature `c_ffi` to be enabled.");
            std::process::exit(1);
        }

    } else {
        let output_file = File::create(output_file_path)?;
        Ok(Box::new(BufWriter::new(output_file)))
    }
}

fn process_all_fasta_and_merge(args: &FastixeArgs, files: &[String], output_file_path: &Path) -> std::io::Result<()> {
    let (sender, receiver) = unbounded();
    files.par_iter().for_each_with(sender.clone(), |s, file_path| {
        if let Ok(results) = process_fasta_needle(file_path.as_ref(), &args.reg, args.uppercase) {
            for line in results {
                s.send(line).expect("Failed to send!");
            }
        }

    });
    drop(sender);

    let mut writer = create_all_fasta_and_merge_writer(output_file_path, args.merge_bgzip_output, args.compression_level, args.threads)?;

    for line in receiver {
        writer.write_all(line.as_bytes())?;
    }

    Ok(())
} 

fn process_all_fasta(args: &FastixeArgs, input_genomes: &[String]) -> std::io::Result<()> {
    if input_genomes.len() > 1 {
        input_genomes.par_iter().try_for_each(|input_genome| {
            let output_genome = make_output_path(input_genome.as_ref(), &args.out_directory, args.gzip_output);
            let prefix = extract_prefix_from_path(input_genome.as_ref(), args.reg.as_ref())?;
            process_fasta(input_genome.as_ref(), &output_genome, &prefix, args.uppercase, args.gzip_output, args.compression_level, args.is_stdout)
        })?;
    } else {
        let input_genomes_first = input_genomes.first().unwrap().as_ref();
        let output_genome = if input_genomes_first == Path::new("-") {
            PathBuf::from("-") 
        } else {
            make_output_path(input_genomes_first, &args.out_directory, args.gzip_output)
        };
        if args.prefix.is_some() {
            process_fasta(input_genomes_first, &output_genome, &args.prefix.as_ref().unwrap(), args.uppercase, args.gzip_output, args.compression_level, args.is_stdout)?;
        } else {
            let prefix = extract_prefix_from_path(input_genomes_first, args.reg.as_ref())?;
            process_fasta(input_genomes_first, &output_genome, prefix.as_str(), args.uppercase, args.gzip_output, args.compression_level, args.is_stdout)?;
        }
    }
    Ok(())
}

pub fn fastixe(args: FastixeArgs) -> std::io::Result<()> {
    let mut input_genomes = vec![];

    check_args_valid(&args);
    parse_files(&args, &mut input_genomes);
    create_dir_all(&args.out_directory)?;
    // println!("input genomes: {:?}", input_genomes);
    if args.merge_output {
        let mut merged_path = Path::new(&args.out_directory).join(&args.merge_output_file_path);
        if args.merge_bgzip_output {
            merged_path.set_extension("gz");
        };
        process_all_fasta_and_merge(&args,&input_genomes, &merged_path)?;
        if args.faidx {
            #[cfg(feature = "c_ffi")]
            build(&merged_path).expect("Failed to build FASTA index");

            #[cfg(not(feature = "c_ffi"))]
            {
                eprintln!("Error: `--faidx` requires feature `c_ffi` to be enabled.");
                std::process::exit(1);
            }        
        }
    } else {
        process_all_fasta(&args, &input_genomes)?;
    }

    Ok(())
}
