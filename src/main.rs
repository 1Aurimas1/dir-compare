use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, fs, io, path::Path, ptr, time::Instant};

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

#[derive(Serialize, Deserialize)]
struct Duplicate<'a> {
    file_name: &'a str,
    first_dir_match: &'a str,
    second_dir_match: Vec<String>,
}

fn main() -> io::Result<()> {
    let start_all = Instant::now();
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() < 1 {
        panic!("Provide 1 or 2 arguments...");
    }

    let mut dirs: Vec<Dir> = Vec::new();
    //for arg in args.iter().skip(1) {
    for arg in args.iter() {
        // only 2 no need to iter all
        let dir = Path::new(arg);
        let mut files: Vec<File> = Vec::new();
        let now = Instant::now();
        if let Err(e) = walk_dir(dir, &mut files) {
            panic!("Couldn't read directory. Error: {}", e);
        } else {
            dirs.push(Dir::new(arg.to_string(), files));
        }
        let elapsed = now.elapsed();
        println!("Dir walker time: {:.2?}, dir: {}", elapsed, arg);
    }

    if dirs.len() == 0 {
        eprintln!("No dirs found");
    } else {
        let now = Instant::now();

        let duplicates2 = find_duplicates(&dirs[0].files, &dirs[dirs.len() - 1].files);

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

fn walk_dir(dir: &Path, files: &mut Vec<File>) -> io::Result<()> {
    let entries = fs::read_dir(dir);
    if let Err(e) = entries {
        eprintln!("Err {:?}, Dir {:?}", e, dir);
        return Ok(());
    }
    for entry in entries? {
        let entry = entry?;
        let path = entry.path();

        if entry.file_type()?.is_dir() {
            walk_dir(&path, files)?;
        } else {
            let size = entry.metadata()?.len();
            //let size = 0;
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

    Ok(())
}

fn find_duplicates<'a>(files1: &'a Vec<File>, files2: &'a Vec<File>) -> Vec<Duplicate<'a>> {
    let mut matches: HashMap<&String, Duplicate> = if !ptr::eq(files1, files2) {
        files1
            .into_iter()
            .map(|file| {
                (
                    &file.name,
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
        if let Some(dups) = matches.get_mut(&file.name) {
            dups.second_dir_match.push(file.path.clone());
        } else {
            matches.insert(
                &file.name,
                Duplicate {
                    file_name: &file.name,
                    first_dir_match: &file.path,
                    second_dir_match: Vec::new(),
                },
            );
        }
    }

    matches
        .into_values()
        .filter(|m| !m.second_dir_match.is_empty())
        .collect()
}
