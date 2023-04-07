use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, fs, io, path::Path, ptr};

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
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.len() > 3 {
        panic!("Provide 1 or 2 arguments...");
    }

    let mut dirs: Vec<Dir> = Vec::new();
    for arg in args.iter().skip(1) {
        let dir = Path::new(arg);
        let mut files: Vec<File> = Vec::new();
        if let Err(e) = walk_dir(dir, &mut files) {
            panic!("Couldn't read directory. Error: {}", e);
        } else {
            dirs.push(Dir::new(arg.to_string(), files));
        }
    }

    if dirs.len() == 0 {
        eprintln!("No dirs found");
    } else {
        let duplicates2 = find_duplicates(&dirs[0].files, &dirs[dirs.len() - 1].files);
        let serialized = serde_json::to_string_pretty(&duplicates2)?;
        println!("{:}", serialized);

        let output_file = "./duplicates.json";
        fs::write(output_file, serialized)?; // return
    };

    Ok(())
}

fn walk_dir(dir: &Path, files: &mut Vec<File>) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::metadata(&path)?;

        if metadata.is_dir() {
            walk_dir(&path, files)?;
        } else {
            files.push(File {
                name: String::from(entry.file_name().to_str().unwrap()),
                path: String::from(path.to_str().unwrap()),
                size: metadata.len(),
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
