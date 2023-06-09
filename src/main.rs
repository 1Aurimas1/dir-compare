use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    fs::{self, read_dir},
    io,
    path::{Path, PathBuf},
    ptr,
    time::Instant,
};

#[derive(Debug, Eq, Hash, Clone, Serialize, Deserialize)]
struct File {
    name: String,
    path: String,
    size: u64,
}

impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

struct Dir {
    name: String,
    files: Vec<File>,
}

impl Dir {
    fn new(name: String, files: Vec<File>) -> Self {
        Self { name, files }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Duplicate<'a> {
    file_name: &'a str,
    first_dir_match: &'a str,
    second_dir_match: Vec<String>,
}

fn parse_args() -> Result<Vec<PathBuf>, String> {
    let args: Vec<String> = env::args().skip(1).take(2).collect();

    if args.is_empty() {
        //panic!("Provide 1 or 2 arguments...");
        return Err("Provide 1 or 2 arguments...".into());
    }

    args.into_iter()
        .map(|arg| {
            let p = PathBuf::from(arg);
            if !p.exists() {
                Err(format!("Path does not exist: {}", p.display()))
            } else if !p.is_dir() {
                Err(format!("Path is not a dir: {}", p.display()))
            } else {
                Ok(p)
            }
        })
        .collect()
}

fn read_dirs(paths: Vec<PathBuf>) -> Result<Vec<Dir>, String> {
    //let mut dirs: Vec<Dir> = Vec::new();
    //for arg in args.iter().skip(1)
    //for path in paths.iter() {
    //}
    paths
        .into_iter()
        .map(|path| {
            //let dir = Path::new(path);
            let mut files: Vec<File> = Vec::new();
            //let now = Instant::now();
            if let Err(e) = walk_dir(&path, &mut files) {
                Err(format!(
                    "Couldn't read directory: {}. Error: {}",
                    path.display(),
                    e
                ))
            } else {
                //dirs.push(Dir::new(path.display().to_string(), files)); // could improve? to_string
                //let elapsed = now.elapsed();
                //println!("Dir walker time: {:.2?}, dir: {}", elapsed, path.display());
                // check files empty?
                Ok(Dir::new(path.display().to_string(), files)) // could improve? to_string
            }
        })
        .collect()
}

fn walk_dir(dir: &Path, files: &mut Vec<File>) -> Result<(), Box<dyn std::error::Error>> {
    let entries = fs::read_dir(dir);
    // just testing
    //if let Err(e) = entries {
    //    eprintln!("Err {:?}, Dir {:?}", e, dir);
    //    return Ok(());
    //}
    let entries = match entries {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Err {:?}, Dir {:?}", e, dir);
            return Ok(());
        }
    };
    for entry in entries {
        //let entry = entry?;
        if let Ok(entry) = entry {
            let path = entry.path();

            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    //println!("{:?}", entry);
                    walk_dir(&path, files)?;
                } else {
                    // metadata...unwrap? with default value instead
                    let size = match entry.metadata() {
                        Ok(metadata) => metadata.len(),
                        Err(e) => {
                            eprintln!("Metadata size err: {}", e);
                            0
                        }
                    };
                    //let size = match std::fs::metadata(entry.path()) {
                    //    Ok(metadata) => metadata.len(),
                    //    Err(e) => {
                    //        eprintln!("Metadata size err: {}", e);
                    //        0
                    //    }
                    //};
                    //let size = std::fs::metadata(entry.path())?.len();
                    //std::fs::metadata(entry.path());
                    //let size = 0;
                    if entry.file_name() == "tap.lua" {
                        println!("{:?}", path);
                    }
                    files.push(File {
                        name: entry
                            .file_name()
                            .into_string()
                            .expect("Invalid Unicode data"),
                        path: path.to_str().expect("Invalid Unicode data").to_string(),
                        size,
                    });
                }
            }
        }
    }

    Ok(())
}

fn find_duplicates<'a>(files1: &'a Vec<File>, files2: &'a Vec<File>) -> Vec<Duplicate<'a>> {
    let mut matches: HashMap<String, Duplicate> = if !ptr::eq(files1, files2) {
        files1
            .into_iter()
            .map(|file| {
                (
                    file.name.clone(),
                    Duplicate {
                        file_name: &file.name,
                        first_dir_match: &file.path,
                        second_dir_match: Vec::new(),
                    },
                )
            })
            .collect()
    } else {
        HashMap::new()
    };

    for file in files2 {
        match matches.get_mut(&file.name) {
            Some(dups) => dups.second_dir_match.push(file.path.clone()),
            None => {
                matches.insert(
                    file.name.clone(),
                    Duplicate {
                        file_name: &file.name,
                        first_dir_match: &file.path,
                        second_dir_match: Vec::new(),
                    },
                );
            }
        }
    }

    matches
        .into_values()
        .filter(|m| !m.second_dir_match.is_empty())
        .collect()
}

fn entry() -> Result<(), Box<dyn std::error::Error>> {
    let start_all = Instant::now();

    let paths = parse_args()?;

    let now = Instant::now();
    let dirs = read_dirs(paths)?;
    let elapsed = now.elapsed();
    println!("Dir walker time: {:.2?}", elapsed);

    if dirs.len() == 0 {
        // unnecessary?
        return Err("No dirs found".into());
    } else {
        let now = Instant::now();

        let duplicates2 = find_duplicates(&dirs[0].files, &dirs.get(1).unwrap_or(&dirs[0]).files);

        let elapsed = now.elapsed();
        println!("Duplicate finder time: {:.2?}", elapsed);

        let serialized = serde_json::to_string_pretty(&duplicates2)?;

        println!("First folder total duplicates: {:?}", duplicates2.len());
        println!(
            "Second folder total duplicates: {:?}",
            duplicates2
                .iter()
                .fold(0, |acc, dup| acc + dup.second_dir_match.len())
        );

        let output_file = "./duplicates.json";
        fs::write(output_file, serialized)?; // return
    };

    let elapsed = start_all.elapsed();
    println!("Total time: {:.2?}", elapsed);
    Ok(())
}

fn main() {
    match entry() {
        Ok(()) => (),
        Err(e) => eprintln!("error: {}", e),
    }
}
