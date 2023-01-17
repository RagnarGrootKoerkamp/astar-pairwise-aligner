use bio::io::fasta::{self, IndexedReader};
use clap::Parser;
use itertools::Itertools;
use std::{io::Write, path::PathBuf};

#[derive(Parser)]
#[clap(
    name = "NanoSim to .seq",
    about = "Given NanoSim .fasta output, extract the corresponding reference parts by using sequence header information.",
    author = "Ragnar Groot Koerkamp"
)]
struct Cli {
    // Where to write the output .seq.
    #[clap(short, long, value_parser = clap::value_parser!(PathBuf))]
    output: PathBuf,

    // The reference.
    #[clap(long, value_parser = clap::value_parser!(PathBuf))]
    reference: PathBuf,

    // The NanoSim samples.
    #[clap(long, value_parser = clap::value_parser!(PathBuf))]
    reads: PathBuf,

    // The number of reads to keep.
    #[clap(short = 'n', long)]
    count: Option<usize>,

    // Whether to strip head/tail unaligned regions
    #[clap(long)]
    strip_unaligned: bool,
}

fn main() {
    let args = Cli::parse();

    assert_eq!(args.output.extension().unwrap_or_default(), "seq");

    let mut index = IndexedReader::from_file(&args.reference)
        .expect("Could not read reference or reference.fai");

    let mut reads = fasta::Reader::from_file(args.reads)
        .expect("Could not read reads")
        .records();

    let mut output = std::fs::File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(args.output)
        .unwrap();

    let mut count = 0;
    while let Some(Ok(record)) = reads.next() {
        // Example header format:
        // chr4_92140456_aligned_0_F_9_1147053_14
        // name_start   _aligned_id_F/R_
        // 9_1147053_14: 1147053 bases from the reference, with 9 prepended and 14 appended.
        let header = record.id();
        let parts = header.split('_').collect_vec();
        if let [name, start, _aligned, id, strand, prefix, length, suffix] = parts[..] {
            print!("{header:<20}:\t");
            assert!(strand == "F");
            let start: usize = start.parse().unwrap();
            let prefix: usize = prefix.parse().unwrap();
            let length: usize = length.parse().unwrap();
            let suffix: usize = suffix.parse().unwrap();
            index
                .fetch(name, start as u64, (start + length) as u64)
                .expect("Could not find chromosome in index.");
            let mut aligned_read = Vec::new();
            index
                .read(&mut aligned_read)
                .expect("Could not read the interval from the reference");

            if aligned_read.contains(&('N' as u8)) {
                println!("Found N chars in reference for read {id}. Dropping read.");
                continue;
            } else {
                println!("Writing read {id}.");
            }

            let read = if args.strip_unaligned {
                let seq = record.seq();
                match (&seq).get(prefix..seq.len() - suffix) {
                    Some(subset) => subset.to_vec(),
                    None => {
                        println!("could not take subsequence: {prefix}..{length}-{suffix} of seq of len {}.", seq.len());
                        continue;
                    }
                }
            } else {
                record.seq().to_vec()
            };

            // First write the reference, and then the read.
            output.write(">".as_bytes()).unwrap();
            output.write(&aligned_read).unwrap();
            output.write("\n".as_bytes()).unwrap();
            output.write("<".as_bytes()).unwrap();
            output.write(&read).unwrap();
            output.write("\n".as_bytes()).unwrap();

            count += 1;
            if Some(count) == args.count {
                break;
            }
        } else {
            panic!("Header does not have the right format");
        }
    }
}
