use chrono::{DateTime, Local, Utc};
use colored::Colorize;
use dirs::home_dir;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::time::{Duration, SystemTime};
use walkdir::WalkDir;

use clap::{ArgGroup, Parser};

const PLAYLISTS_FOLDER: &str = "playlists/";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group(
    ArgGroup::new("mode")
        .required(true)
        .args(["threshold_days", "detect"]),
))]
struct Cli {
    /// Enable debug output
    #[arg(long)]
    debug: bool,

    /// Do not write to file (dry-run)
    #[arg(long)]
    dryrun: bool,

    /// Sets the root search directory
    #[arg(short, long)]
    directory: String,

    /// Sets the path to the playlist file
    #[arg(short, long)]
    playlist: String,

    /// Sets the threshold in days
    #[arg(short, long, conflicts_with = "detect", required_unless_present = "detect")]
    threshold_days: Option<u32>,

    /// Auto-detect day threshold based on .last_run
    #[arg(long, conflicts_with = "threshold_days", required_unless_present = "threshold_days")]
    detect: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if !cli.detect && cli.threshold_days.is_none() {
        return Err("Error: Either --detect or --threshold-days must be provided.".into());
    }

    let now = SystemTime::now();
    let last_run_timestamp = read_last_run_timestamp().unwrap_or(None);

    if let Some(last_run) = last_run_timestamp {
        let difference = now.duration_since(last_run).unwrap_or(Duration::from_secs(0));
        let days_since_last_run = difference.as_secs() / (24 * 60 * 60);

        println!(
            "{} {}",
            "[-] Last run timestamp:".cyan(),
            format_system_time(last_run).cyan().bold()
        );
        println!(
            "{}       {}",
            "[+] Days since last run:".cyan(),
            days_since_last_run.to_string().cyan().bold()
        );
    } else {
        println!("{}", "[-] No last run timestamp found.".yellow());
    }

    let threshold_time = if cli.detect {
        let last_run = last_run_timestamp.ok_or("Error: --detect used without a valid last run timestamp.")?;
        println!("{}", "[+] Using last_run timestamp directly...".yellow());
        last_run
    } else {
        let days = cli.threshold_days.unwrap();
        println!(
            "{} {}",
            "[+] Using Threshold (in days):".red(),
            days.to_string().red().bold()
        );
        now - Duration::from_secs(days as u64 * 24 * 60 * 60)
    };

    println!(
        "{}    {}",
        "[-] Threshold time:".red(),
        format_system_time(threshold_time).red().bold()
    );

    let mut playlist = Vec::new();
    process_directory(&cli.directory, threshold_time, &mut playlist, cli.debug)?;

    if cli.dryrun {
        println!("{}", "[!] dry-run, doing no work...".yellow());
        for item in playlist {
            println!("{}", item);
        }
    } else {
        write_playlist(&cli.playlist, &playlist)?;
        if !cli.dryrun {
            save_last_run_timestamp()?;
        }
        println!(
            "{} {}",
            "[+] Playlist file successfully updated at:".green(),
            cli.playlist.green().bold()
        );
    }

    Ok(())
}

fn process_directory(root_path: &str, threshold_time: SystemTime, playlist: &mut Vec<String>, debug: bool) -> io::Result<()> {
    let canonical_root = fs::canonicalize(root_path)?;
    let playlists_path = canonical_root.join(PLAYLISTS_FOLDER);

    for entry in WalkDir::new(root_path)
        .into_iter()
        .filter_entry(|e| {
            fs::canonicalize(e.path())
                .map(|p| p != playlists_path)
                .unwrap_or(true)
        })
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.metadata().map(|m| {
                m.is_file() && m.modified().is_ok_and(|mod_time| mod_time > threshold_time)
            }).unwrap_or(false) && !e.path().starts_with(&playlists_path)
        })
    {
        let path = entry.path();
        if let Ok(relative_path) = path.strip_prefix(root_path) {
            if let Some(path_str) = relative_path.to_str() {
                if debug {
                    println!("Found: {}", path_str);
                }
                playlist.push(path_str.to_string());
            }
        }
    }
    Ok(())
}

fn write_playlist(playlist_path: &str, playlist: &[String]) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(playlist_path)?;

    for item in playlist {
        writeln!(file, "{}", item)?;
    }
    Ok(())
}

fn save_last_run_timestamp() -> io::Result<()> {
    if let Some(mut home_dir) = home_dir() {
        home_dir.push(".jukeingest");
        fs::create_dir_all(&home_dir)?;
        home_dir.push("last_run");

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_secs();

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(home_dir)?;

        write!(file, "{}", timestamp)?;
        return Ok(());
    }
    Err(io::Error::other("Failed to determine home directory"))
}

fn read_last_run_timestamp() -> io::Result<Option<SystemTime>> {
    if let Some(mut home_dir) = home_dir() {
        home_dir.push(".jukeingest");
        home_dir.push("last_run");

        if let Ok(mut file) = File::open(&home_dir) {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            if let Ok(parsed_secs) = contents.trim().parse::<u64>() {
                return Ok(Some(SystemTime::UNIX_EPOCH + Duration::from_secs(parsed_secs)));
            }
        }
    }
    Ok(None)
}

fn format_system_time(system_time: SystemTime) -> String {
    let datetime: DateTime<Utc> = system_time.into();
    datetime
        .with_timezone(&Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_process_directory_excludes_playlists() -> io::Result<()> {
        let dir = tempdir()?;
        let root_path = dir.path();

        let playlists_dir = root_path.join("playlists");
        fs::create_dir(&playlists_dir)?;

        let file1 = root_path.join("song1.mp3");
        fs::write(&file1, "audio")?;

        let file2 = playlists_dir.join("excluded.m3u");
        fs::write(&file2, "playlist")?;

        let mut playlist = Vec::new();
        let threshold_time = SystemTime::UNIX_EPOCH; // Include everything

        process_directory(root_path.to_str().unwrap(), threshold_time, &mut playlist, false)?;

        assert!(playlist.iter().any(|p| p.ends_with("song1.mp3")));
        assert!(!playlist.iter().any(|p| p.contains("excluded.m3u")));
        assert_eq!(playlist.len(), 1);

        Ok(())
    }
}
