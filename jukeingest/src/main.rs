use chrono::{DateTime, Local, Utc};
use colored::Colorize;
use dirs::home_dir;
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

    // TODO: these two need to conflict
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
    let detect = &cli.detect;

    let now = SystemTime::now();

    // last run logic
    let mut calcd_threshold_days: u64 = 0;
    let last_run_timestamp = read_last_run_timestamp().unwrap_or(None);

    if let Some(last_run) = last_run_timestamp {
        let difference = now
            .duration_since(last_run)
            .unwrap_or(Duration::from_secs(0));
        calcd_threshold_days = difference.as_secs() / (24 * 60 * 60); // Convert seconds to days

	println!("{}", format!("[-] Last run timestamp: {}", format_system_time(last_run)).cyan());
        println!("{} {}", format!("[+] Calculated Threshold (in days):").cyan(), format!("{}", calcd_threshold_days).cyan().bold());
    } else {
        println!("{}", "[-] No last run timestamp found.".yellow());
    }

    // actually set the timestamp based on CLI args
    let mut threshold_time: SystemTime = now;
    if *detect {
        threshold_time = now - Duration::from_secs((calcd_threshold_days * 24 * 60 * 60).into());
    } else {
        threshold_time = now - Duration::from_secs((threshold_days * 24 * 60 * 60).into());
    }

    println!("{}", format_system_time(threshold_time));

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
        eprintln!("{}", format!("[!] dry-run, doing no work...").yellow());
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

    let _ = playlist_stdout.flush();

    println!("{} {}", format!("[+] Playlist file successfully updated at:").green(), format!("{}", playlist_path).green().bold());
    println!("{}", format!("[-] writing last_run timestamp file ...").green());

    save_last_run_timestamp().unwrap_or_else(|err| {
        eprintln!("Error saving last run timestamp: {}", err);
    });
}

fn save_last_run_timestamp() -> io::Result<()> {
    // Get the user's home directory
    if let Some(mut home_dir) = home_dir() {
        home_dir.push(".last_run");

        let timestamp = SystemTime::now();

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(home_dir)?;

        // Use map_err to convert SystemTimeError to io::Error
        write!(file, "{}", timestamp.duration_since(SystemTime::UNIX_EPOCH).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?.as_secs())?;

        return Ok(());
    }

    Err(io::Error::new(io::ErrorKind::Other, "Failed to determine home directory"))
}

fn read_last_run_timestamp() -> io::Result<Option<SystemTime>> {
    // Get the user's home directory
    if let Some(mut home_dir) = home_dir() {
        home_dir.push(".last_run");

        if let Ok(mut file) = File::open(home_dir) {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            if let Ok(parsed_date) = contents.trim().parse::<u64>() {
                let timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(parsed_date);
                return Ok(Some(timestamp));
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
