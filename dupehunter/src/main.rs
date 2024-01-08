use std::collections::HashMap;
use std::fs;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Get the current directory
    let current_dir = env::current_dir()?;
    
    // Create a HashMap to store file names in a case-insensitive manner
    let mut file_names: HashMap<String, Vec<String>> = HashMap::new();

    // Iterate over the entries in the directory
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_lowercase();

        // Check if the file name already exists in the HashMap
        let entry_path = entry.path().display().to_string();
        let entry_paths = file_names.entry(file_name.into()).or_insert(vec![]);
        entry_paths.push(entry_path);
    }

    // Print the files with case-insensitive collisions
    for (file_name, paths) in file_names.iter().filter(|(_, paths)| paths.len() > 1) {
        println!("Collisions for file name '{}':", file_name);
        for path in paths {
            println!("  - {}", path);
        }
    }

    Ok(())
}
