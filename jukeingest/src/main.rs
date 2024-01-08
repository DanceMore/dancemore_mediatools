use chrono::{DateTime, Local, Utc};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::time::{Duration, SystemTime};
use walkdir::WalkDir;

use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    /// Enable debug output
    #[arg(long)]
    debug: bool,

    /// do not write to file (dry-run)
    #[arg(long)]
    dryrun: bool,

    /// Sets the root search directory
    #[clap(short, long)]
    directory: String,

    /// Sets the path to the playlist file
    #[clap(short, long)]
    playlist: String,

    /// Sets the threshold in days
    #[arg(short, long, default_value = "30")]
    threshold_days: u32,

    /// auto detect day threshold based off .last_run
    #[arg(long)]
    detect: bool,
}

fn main() {
    let cli: Cli = Cli::parse();

    let debug = &cli.debug;
    let dryrun = &cli.dryrun;
    let root_path = &cli.directory;
    let playlist_path = &cli.playlist;
    let threshold_days = &cli.threshold_days;

    let now = SystemTime::now();
    let threshold_time = now - Duration::from_secs((threshold_days * 24 * 60 * 60).into());

    // last run logic:w

    let last_run_timestamp = read_last_run_timestamp().unwrap_or(None);

    if let Some(last_run) = last_run_timestamp {
        let difference = now
            .duration_since(last_run)
            .unwrap_or(Duration::from_secs(0));
        let calcd_threshold_days = difference.as_secs() / (24 * 60 * 60); // Convert seconds to days

        println!("Last run timestamp: {}", format_system_time(last_run));
        println!("Threshold days: {}", calcd_threshold_days);
    } else {
        println!("No last run timestamp found.");
    }
    save_last_run_timestamp().unwrap_or_else(|err| {
        eprintln!("Error saving last run timestamp: {}", err);
    });

    // build it for append-only
    let mut playlist_file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(playlist_path)
    {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Error opening playlist file: {}", err);
            return;
        }
    };

    let stdout = io::stdout();
    let mut playlist_stdout = io::BufWriter::new(stdout.lock());

    if *dryrun {
        eprintln!("[!] dry-run, doing no work...");
    }

    for entry in WalkDir::new(root_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .metadata()
                .map(|m| {
                    m.is_file()
                        && m.modified()
                            .ok()
                            .map_or(false, |mod_time| mod_time > threshold_time)
                })
                .unwrap_or(false)
        })
    {
        let path = entry.path();
        if let Ok(Some(file_name)) = path.strip_prefix(root_path).and_then(|p| Ok(p.to_str())) {
            if *debug {
                if let Err(err) = writeln!(playlist_stdout, "{}", file_name) {
                    eprintln!("Error writing to stdout: {}", err);
                }
            }

            if !(*dryrun) {
                if let Err(err) = writeln!(playlist_file, "{}", file_name) {
                    eprintln!("Error writing to playlist file: {}", err);
                }
            }
        } else {
            eprintln!("Error stripping prefix for path: {:?}", path);
        }
    }

    println!("Playlist file successfully updated at: {}", playlist_path);
}

fn save_last_run_timestamp() -> io::Result<()> {
    let last_run_path = ".last_run";
    let timestamp = SystemTime::now();

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(last_run_path)?;

    // Use map_err to convert SystemTimeError to io::Error
    write!(
        file,
        "{}",
        timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
            .as_secs()
    )?;

    Ok(())
}

fn read_last_run_timestamp() -> io::Result<Option<SystemTime>> {
    let last_run_path = ".last_run";

    if let Ok(mut file) = File::open(last_run_path) {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        if let Ok(timestamp) = contents.trim().parse::<u64>() {
            return Ok(Some(
                SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp),
            ));
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
