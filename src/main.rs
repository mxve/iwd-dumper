use clap::Parser;
use rayon::prelude::*;
use std::{fs, path::Path};
use walkdir::WalkDir;
use colored::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Source directory
    #[clap(short, long, required = true)]
    source: String,

    /// Output directory
    #[clap(short, long, required = true)]
    output: String,

    /// Only dump this file extension
    #[clap(short, long, default_value = "all")]
    extension: String,

    /// Concurrent threads to use (0 = system threads)
    #[clap(short, long, default_value_t = 0)]
    threads: usize,
}

macro_rules! skip_fail {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                println!("An error: {}; skipped.", e);
                continue;
            }
        }
    };
}

pub fn unzip(zip_path: &Path, out_path: &Path, extension: &String) {
    let mut archive = zip::ZipArchive::new(fs::File::open(zip_path).unwrap()).unwrap();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if extension != "all" && !file.name().ends_with(extension) {
            continue;
        }
        let outpath = out_path.join(file.name());

        if file.name().ends_with('/') && !outpath.exists() {
            skip_fail!(fs::create_dir_all(outpath));
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    skip_fail!(fs::create_dir_all(p));
                }
            }
            let Ok(mut outfile) = fs::File::create(&outpath) else {
                continue;
            };
            skip_fail!(std::io::copy(&mut file, &mut outfile));
        }
    }
}

fn main() {
    let args = Args::parse();

    rayon::ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build_global()
        .unwrap();

    println!(
            "Dumping\n    IWDs from\n        {}\n    to\n        {}\n    threads: {}\n    extension: {}",
            args.source, args.output, rayon::current_num_threads(), args.extension
        );

    let file_paths: Vec<_> = WalkDir::new(Path::new(&args.source))
        .into_iter()
        .filter_map(|e| e.ok())
        .par_bridge()
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            if let Some(extension) = entry.path().extension() {
                if extension == "iwd" {
                    Some(entry.path().to_owned())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    file_paths.into_par_iter().for_each(|file_path| {
        println!("[{}] {}", "Starting".bright_purple(), file_path.display());
        unzip(
            file_path.as_path(),
            Path::new(&args.output),
            &args.extension,
        );
        println!("[{}] {}", "Finished".bright_green(), file_path.display());
    });
}
