use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, Duration};
use walkdir::WalkDir;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// Sets the root search directory
    #[clap(short, long)]
    directory: String,

    /// Sets the path to the playlist file
    #[clap(short, long)]
    playlist: String,

    /// Sets the threshold in days
    #[clap(short, long, default_value = "30")]
    threshold_days: u32,
}

fn main() {
    let args: Args = Args::parse();

    let root_path = &args.directory;
    let playlist_path = &args.playlist;
    let threshold_days = args.threshold_days;

    let now = SystemTime::now();
    let threshold_time = now - Duration::from_secs((threshold_days * 24 * 60 * 60).into());

    let entries = WalkDir::new(root_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.metadata()
                .map(|m| m.is_file() && m.modified().ok().is_some())
                .unwrap_or(false)
        });

    let mut playlist_content = String::new();

    let playlist_content: String = entries
	    .map(|entry| entry.path().to_string_lossy().to_string())
	    .collect::<Vec<_>>()
	    .join("\n");

    if !playlist_content.is_empty() {
        if let Err(err) = append_to_playlist_file(playlist_path, playlist_content) {
            eprintln!("Error appending to playlist file: {}", err);
        } else {
            println!("Playlist file successfully updated at: {}", playlist_path);
        }
    } else {
        println!("No files found to add to the playlist.");
    }
}

fn append_to_playlist_file(file_path: &str, content: String) -> io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(file_path)?;
    writeln!(file, "{}", content)?;
    Ok(())
}
