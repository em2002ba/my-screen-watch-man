use clap::{Parser, Subcommand};
use chrono::Local;
use std::{fs, path::PathBuf};

const DATA_FILE: &str = "usage_log.json";
const CONFIG_FILE: &str = "config.json";

#[derive(Parser)]
#[command(name = "ScreenWatchMan")]
#[command(about = "Track your phone screen time and get sleep alerts", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Log screen time in minutes
    Log {
        minutes: u32,
    },
    /// Show today's summary
    Summary,
    /// Set your sleep and wake time
    Set {
        #[arg(long)]
        sleep: String,

        #[arg(long)]
        wake: String,
    },
    /// Check if it's bedtime or too early
    Check,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct DailyUsage {
    date: String,
    total_minutes: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Config {
    sleep_time: String,
    wake_time: String,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Log { minutes } => {
            log_time(*minutes);
        }
        Commands::Summary => {
            show_summary();
        }
        Commands::Set { sleep, wake } => {
            set_config(sleep, wake);
        }
        Commands::Check => {
            check_time();
        }
    }
}

fn log_time(minutes: u32) {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let mut log = load_usage();

    let usage = log.iter_mut().find(|entry| entry.date == today);
    if let Some(entry) = usage {
        entry.total_minutes += minutes;
    } else {
        log.push(DailyUsage {
            date: today.clone(),
            total_minutes: minutes,
        });
    }

    save_usage(&log);
    println!(
        "✅ Logged {} minutes. Total today: {} mins",
        minutes,
        log.iter().find(|e| e.date == today).unwrap().total_minutes
    );
}

fn show_summary() {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let log = load_usage();

    if let Some(entry) = log.iter().find(|entry| entry.date == today) {
        println!(
            "📊 Today ({}) you’ve spent {} minutes on your phone.",
            entry.date, entry.total_minutes
        );
    } else {
        println!("📭 No screen time logged for today.");
    }
}

fn load_usage() -> Vec<DailyUsage> {
    if PathBuf::from(DATA_FILE).exists() {
        let data = fs::read_to_string(DATA_FILE).expect("Failed to read usage log");
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    }
}

fn save_usage(log: &Vec<DailyUsage>) {
    let json = serde_json::to_string_pretty(log).expect("Failed to serialize usage log");
    fs::write(DATA_FILE, json).expect("Failed to write usage log");
}

fn set_config(sleep: &str, wake: &str) {
    let config = Config {
        sleep_time: sleep.to_string(),
        wake_time: wake.to_string(),
    };

    if let Ok(json) = serde_json::to_string_pretty(&config) {
        fs::write(CONFIG_FILE, json).expect("Failed to save config");
        println!("🛏️ Sleep time set to: {}", sleep);
        println!("⏰ Wake time set to: {}", wake);
    }
}

fn check_time() {
    let now = Local::now().format("%H:%M").to_string();

    if let Ok(config_data) = fs::read_to_string(CONFIG_FILE) {
        if let Ok(config) = serde_json::from_str::<Config>(&config_data) {
            let sleep_time = &config.sleep_time;
            let wake_time = &config.wake_time;

            if now >= *sleep_time || now < *wake_time {
                println!(
                    "🚫 It's {} now — that's outside your healthy screen hours ({} → {})",
                    now, sleep_time, wake_time
                );
            } else {
                println!(
                    "✅ You're within your screen time window. It's currently {}",
                    now
                );
            }
        } else {
            println!("⚠️ Couldn't parse config file.");
        }
    } else {
        println!("⚠️ No config file found. Use the `set` command first.");
    }
}
