use std::collections::HashMap;
use std::fs::{read, read_dir, DirEntry};
use std::path::{Path, PathBuf};
use std::process::exit;

use blake2::{Blake2s, Digest};
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;

fn browse_dir<P: AsRef<Path>>(paths: &[P]) -> Vec<DirEntry> {
    paths
        .into_iter()
        .flat_map(|path| {
            read_dir(path)
                .unwrap()
                .filter_map(|f| f.ok())
                .flat_map(|file| {
                    if file.path().is_dir() {
                        browse_dir(&[file.path()])
                    } else {
                        vec![file]
                    }
                })
        })
        .collect()
}

fn main() {
    let paths: Vec<String> = std::env::args()
        .skip(1)
        .filter(|p| Path::new(p).exists())
        .collect();

    if paths.is_empty() {
        println!("Scan for duplicates files under selected directories");
        println!("Usage : undup <path1> [<path2> ...]");
        exit(1);
    }

    let bar = ProgressBar::new(1);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:60.cyan/blue} {pos:>7}/{len:7}\n{msg}"),
    );
    bar.set_message("Scanning subdirectories...");

    let files = browse_dir(&paths);

    bar.set_length(files.len() as u64);

    let grouped: HashMap<String, Vec<(PathBuf, String)>> = files
        .into_iter()
        .filter_map(|file| {
            bar.set_message(&format!(
                "Hashing file {}...",
                file.path().to_str().unwrap()
            ));
            read(file.path()).ok().map(|content| (file.path(), content))
        })
        .map(|(file_path, file_content)| {
            let mut hasher = Blake2s::new();
            hasher.update(file_content);
            let res = hasher.finalize();
            bar.inc(1);
            let res_str = format!("{:?}", res);

            (file_path, res_str)
        })
        .into_group_map_by(|(_path, hash)| hash.clone())
        .into_iter()
        .filter(|(_hash, dups)| dups.len() > 1)
        .collect();

    bar.set_message("Done !");
    bar.finish();

    let nb_dups = grouped.len();

    for (_hash, dups) in grouped {
        println!("Duplicate file found ({} copies) : ", dups.len());
        for (path, _hash) in dups {
            println!("-- {}", path.to_str().unwrap());
        }
    }

    println!("{} total duplicates were found.", nb_dups);
}
