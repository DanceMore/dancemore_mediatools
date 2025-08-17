use rand::prelude::*;
use rocket::tokio;
use rocket::tokio::time::{Duration, Instant};
use std::time::SystemTime;

use crate::app_state::AppState;

// Configuration constants
const SCHEDULER_INTERVAL: Duration = Duration::from_secs(5); // Increased from 1s to 5s
const MAX_CONSECUTIVE_ERRORS: u32 = 5;
const ERROR_BACKOFF_BASE: u64 = 30; // Base backoff in seconds
const MAX_BACKOFF: u64 = 300; // Max backoff of 5 minutes

#[derive(Debug)]
struct SchedulerState {
    consecutive_errors: u32,
    last_error_time: Option<SystemTime>,
    last_success_time: Option<SystemTime>,
}

impl SchedulerState {
    fn new() -> Self {
        Self {
            consecutive_errors: 0,
            last_error_time: None,
            last_success_time: None,
        }
    }

    fn record_success(&mut self) {
        self.consecutive_errors = 0;
        self.last_success_time = Some(SystemTime::now());
        self.last_error_time = None;
    }

    fn record_error(&mut self) {
        self.consecutive_errors += 1;
        self.last_error_time = Some(SystemTime::now());
    }

    fn should_backoff(&self) -> Option<Duration> {
        if self.consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
            let backoff_seconds = std::cmp::min(
                ERROR_BACKOFF_BASE
                    * (2_u64.pow(
                        self.consecutive_errors
                            .saturating_sub(MAX_CONSECUTIVE_ERRORS),
                    )),
                MAX_BACKOFF,
            );
            Some(Duration::from_secs(backoff_seconds))
        } else {
            None
        }
    }
}

pub async fn start_scheduler(app_state: AppState) {
    info!(
        "Starting scheduler with {}s interval",
        SCHEDULER_INTERVAL.as_secs()
    );
    tokio::spawn(scheduler_mainbody(app_state));
}

async fn scheduler_mainbody(app_state: AppState) {
    let mut scheduler_state = SchedulerState::new();
    let mut iteration_count = 0u64;

    loop {
        let start_time = Instant::now();
        iteration_count += 1;

        // Log iteration count periodically instead of every time
        if iteration_count % 12 == 0 {
            // Every minute with 5s intervals
            debug!("Scheduler iteration #{}", iteration_count);
        }

        // Check if we should back off due to consecutive errors
        if let Some(backoff_duration) = scheduler_state.should_backoff() {
            warn!(
                "Backing off for {}s due to {} consecutive errors",
                backoff_duration.as_secs(),
                scheduler_state.consecutive_errors
            );
            tokio::time::sleep(backoff_duration).await;
            continue;
        }

        match process_scheduler_iteration(&app_state).await {
            Ok(action_taken) => {
                scheduler_state.record_success();
                if action_taken {
                    info!("Successfully processed TV mode request");
                }
            }
            Err(e) => {
                scheduler_state.record_error();

                // Only log errors occasionally to prevent spam
                if scheduler_state.consecutive_errors <= 3
                    || scheduler_state.consecutive_errors % 10 == 0
                {
                    error!(
                        "Scheduler error #{}: {} (will retry in {}s)",
                        scheduler_state.consecutive_errors,
                        e,
                        SCHEDULER_INTERVAL.as_secs()
                    );
                }
            }
        }

        // Calculate how long to sleep to maintain consistent interval
        let elapsed = start_time.elapsed();
        if elapsed < SCHEDULER_INTERVAL {
            tokio::time::sleep(SCHEDULER_INTERVAL - elapsed).await;
        } else {
            // If processing took longer than interval, yield briefly
            tokio::task::yield_now().await;
        }
    }
}

async fn process_scheduler_iteration(app_state: &AppState) -> Result<bool, String> {
    // Get TV mode status
    let tv_mode_status = app_state.tv_mode.read().await.clone();

    if !tv_mode_status.active {
        // TV mode is off, nothing to do (only log this occasionally)
        return Ok(false);
    }

    // Check if media is active
    let is_active = {
        let client = app_state.rpc_client.read().await;
        client
            .is_active()
            .await
            .map_err(|e| format!("Failed to check media status: {}", e))?
    };

    if is_active {
        // Something is already playing, no action needed
        return Ok(false);
    }

    // TV mode is active but nothing is playing - time to act!
    debug!("TV mode active but no media playing, selecting content");

    let user = tv_mode_status
        .user
        .ok_or_else(|| "TV mode active but no user specified".to_string())?;

    // Get user's shows
    let shows = app_state.show_mappings.read().await.sorted_shows();
    let user_shows = shows
        .get(&user)
        .ok_or_else(|| format!("User '{}' not found in show mappings", user))?;

    if user_shows.is_empty() {
        return Err(format!("No shows configured for user '{}'", user));
    }

    // Select random show and episode
    let selected_show = select_random_show_name(user_shows)
        .ok_or_else(|| "Failed to select random show".to_string())?;

    debug!("Selected show '{}' for user '{}'", selected_show, user);

    let rpc_client = app_state.rpc_client.read().await;

    let selected_episode = rpc_client
        .select_random_episode_by_title(selected_show)
        .await
        .map_err(|e| format!("Failed to select episode for '{}': {}", selected_show, e))?;

    rpc_client
        .rpc_play(&selected_episode)
        .await
        .map_err(|e| format!("Failed to play episode: {}", e))?;

    info!(
        "Started playing content for user '{}': {}",
        user, selected_show
    );
    Ok(true)
}

fn select_random_show_name<'a>(shows: &'a [String]) -> Option<&'a String> {
    if shows.is_empty() {
        return None;
    }
    shows.choose(&mut rand::thread_rng())
}
