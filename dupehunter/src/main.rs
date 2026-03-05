use clap::Parser;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Directory to scan (default: current directory)
    #[arg(default_value = ".")]
    pub directory: PathBuf,

    /// Scan directories recursively
    #[arg(short, long)]
    pub recursive: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EntryType {
    File,
    Directory,
}

pub struct Scanner {
    pub args: Args,
    // Maps parent directory -> { lowercase_name -> Vec<(original_name, type)> }
    pub collisions: HashMap<PathBuf, HashMap<String, Vec<(String, EntryType)>>>,
}

impl Scanner {
    pub fn new(args: Args) -> Self {
        Self {
            args,
            collisions: HashMap::new(),
        }
    }

    pub fn scan(&mut self) -> Result<(), Box<dyn Error>> {
        let dir = fs::canonicalize(&self.args.directory)?;
        self.scan_internal(&dir)
    }

    fn scan_internal(&mut self, dir: &Path) -> Result<(), Box<dyn Error>> {
        if !dir.exists() || !dir.is_dir() {
            return Err(format!("Directory '{}' does not exist or is not accessible", dir.display()).into());
        }

        let mut current_dir_entries: HashMap<String, Vec<(String, EntryType)>> = HashMap::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let entry_type = if metadata.is_dir() {
                EntryType::Directory
            } else {
                EntryType::File
            };
            
            if let Some(name) = entry.file_name().to_str() {
                let lower_name = name.to_lowercase();
                current_dir_entries.entry(lower_name).or_default().push((name.to_string(), entry_type));
                
                // If it's a directory and we're recursive, dive in
                if metadata.is_dir() && self.args.recursive {
                    self.scan_internal(&entry.path())?;
                }
            }
        }

        // Only store if there are actually collisions in this directory
        let filtered_entries: HashMap<String, Vec<(String, EntryType)>> = current_dir_entries
            .into_iter()
            .filter(|(_, entries)| entries.len() > 1)
            .collect();

        if !filtered_entries.is_empty() {
            self.collisions.insert(dir.to_path_buf(), filtered_entries);
        }

        Ok(())
    }

    pub fn report(&self) {
        if self.collisions.is_empty() {
            println!("No case-insensitive collisions found.");
            return;
        }

        println!("--- Case-Insensitive Collisions Found ---");
        println!("(These will merge or overwrite on Windows/Mac/SMB/NTFS/APFS)");

        let mut sorted_dirs: Vec<_> = self.collisions.keys().collect();
        sorted_dirs.sort();

        for dir in sorted_dirs {
            println!("\nIn directory '{}':", dir.display());
            let dir_collisions = &self.collisions[dir];
            
            let mut sorted_names: Vec<_> = dir_collisions.keys().collect();
            sorted_names.sort();

            for lower in sorted_names {
                let entries = &dir_collisions[lower];
                let contains_dir = entries.iter().any(|(_, t)| *t == EntryType::Directory);
                
                let label = if contains_dir {
                    "[STRUCTURAL COLLISION]"
                } else {
                    "[FILE COLLISION]"
                };

                println!("  {} '{}' matches:", label, lower);
                for (original, etype) in entries {
                    println!("    - {} ({:?})", original, etype);
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut scanner = Scanner::new(args);
    scanner.scan()?;
    scanner.report();
    Ok(())
}
