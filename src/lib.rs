use abi_stable::std_types::{RString, RVec};
use anyrun_plugin::*;
use serde::Deserialize;
use rust_fuzzy_search::fuzzy_compare;
use std::{
    collections::HashMap,
    fs,
    process::{Command, Stdio},
};
use rand::RngCore;

macro_rules! debug_println {
    ($config:expr, $($arg:tt)*) => {
        if $config.debug {
            println!($($arg)*);
        }
    };
}

fn generate_random_id() -> u64 {
    rand::rng().next_u64()
}

#[derive(Deserialize, Debug, Default)]
struct Config {
    #[serde(default)]
    debug: bool,
    shell: String,
    scripts: Vec<Script>
}

#[derive(Deserialize, Debug)]
struct Script {
    prefix: String,
    command: String,
    #[serde(default)]
    cache: bool,
}

struct State {
    config: Config,
    current_matches: Vec<Entry>,
    cached_entries: HashMap<String, Vec<Entry>>,
}

#[derive(Clone, Deserialize, Debug)]
struct Entry {
    #[serde(default = "generate_random_id")]
    id: u64,
    title: String,
    description: Option<String>,
    icon: Option<String>,
    command: Option<String>,
    score: Option<f32>,
}

#[init]
fn init(config_dir: RString) -> State {
    let config = match fs::read_to_string(format!("{}/external-scripts.ron", config_dir)) {
        Ok(content) => ron::from_str(&content).unwrap_or_else(|why| {
            eprintln!("[external-scripts] Failed to parse config: {}", why);
            Config::default()
        }),
        Err(why) => {
            eprintln!(
                "[external-scripts] No config file provided, using default: {}",
                why
            );
            Config::default()
        }
    };

    debug_println!(config, "[external-scripts] Using config: {:?}", config);

    let mut cached_entries: HashMap<String, Vec<Entry>> = HashMap::new();

    // Execute cacheable scripts at initialization
    for script in &config.scripts {
        if script.cache {
            debug_println!(config, "[external-scripts] Caching script with prefix: {}", script.prefix);

            let output = Command::new(&config.shell)
                .arg("-c")
                .arg(script.command.clone())
                .stdout(Stdio::piped())
                .output();

            match output {
                Ok(output) => {
                    match String::from_utf8(output.stdout) {
                        Ok(output_str) => {
                            match serde_json::from_str::<Vec<Entry>>(&output_str) {
                                Ok(entries) => {
                                    debug_println!(config, "[external-scripts] Cached {} entries for prefix '{}'", entries.len(), script.prefix);
                                    cached_entries.insert(script.prefix.clone(), entries);
                                }
                                Err(why) => {
                                    eprintln!("[external-scripts] Failed to parse cached script output for prefix '{}': {}", script.prefix, why);
                                }
                            }
                        }
                        Err(why) => {
                            eprintln!("[external-scripts] Failed to convert script output to UTF-8 for prefix '{}': {}", script.prefix, why);
                        }
                    }
                }
                Err(why) => {
                    eprintln!("[external-scripts] Failed to execute cacheable script with prefix '{}': {}", script.prefix, why);
                }
            }
        }
    }

    State {
        config,
        current_matches: vec![],
        cached_entries,
    }
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "External Scripts".into(),
        icon: "help-about".into(),
    }
}

#[get_matches]
fn get_matches(input: RString, state: &mut State) -> RVec<Match> {
    debug_println!(state.config, "[external-scripts] Querying: {:?}", input);

    let matching_scripts: Vec<&Script> = state
        .config
        .scripts
        .iter()
        .filter(|script| input.starts_with(&script.prefix))
        .collect();

    debug_println!(state.config, "[external-scripts] Using scripts: {:?}", matching_scripts);

    let mut entries: Vec<Entry> = vec![];

    for script in matching_scripts {
        let query = input.replace(&script.prefix, "").trim().to_string();

        let mut script_entries: Vec<Entry> = if script.cache {
            // Use cached results if available
            if let Some(cached) = state.cached_entries.get(&script.prefix) {
                debug_println!(state.config, "[external-scripts] Using cached results for prefix '{}'", script.prefix);
                cached.clone()
            } else {
                debug_println!(state.config, "[external-scripts] Cache miss for prefix '{}', using empty results", script.prefix);
                vec![]
            }
        } else {
            // Execute script dynamically, passing the query as an argument
            let cmd = if query.is_empty() {
                script.command.clone()
            } else {
                // Properly quote the query for shell safety
                let escaped = query.replace("'", "'\\''");
                format!("{} '{}'", script.command.clone(), escaped)
            };

            let output = Command::new(&state.config.shell)
                .arg("-c")
                .arg(cmd)
                .stdout(Stdio::piped())
                .output()
                .unwrap();

            let output = String::from_utf8(output.stdout).unwrap();
            serde_json::from_str(&output).unwrap()
        };

        script_entries.iter_mut().for_each(|e| {
            e.score = Some(e.score.unwrap_or(fuzzy_compare(&e.title, &query)));
        });

        debug_println!(state.config, "[external-scripts] Script: {:?}", script);
        debug_println!(state.config, "[external-scripts] Matching: {:?}", script_entries);

        entries.extend(script_entries);
    }

    entries.sort_by(|a, b| {
        b.score.unwrap_or_default().total_cmp(&a.score.unwrap_or_default())
    });

    state.current_matches = entries.clone();

    entries
        .iter()
        .map(|entry| Match {
            id: Some(entry.id).into(),
            title: entry.title.clone().into(),
            description: entry.description.clone().map(|v| v.into()).into(),
            icon: entry.icon.clone().map(|v| v.into()).into(),
            use_pango: false,
        })
        .collect()
}

#[handler]
fn handler(selection: Match, state: &State) -> HandleResult {
    debug_println!(state.config, "Selected: {:?}", selection);
    if let Some(selected) = state
        .current_matches
        .iter()
        .find(|m| m.id == selection.id.unwrap_or(0))
        && let Some(command) = &selected.command
    {
        debug_println!(state.config, "[external-scripts] Spawning: {:?}", command);

        let _ = Command::new(&state.config.shell)
            .arg("-c")
            .arg(command)
            .spawn()
            .unwrap()
            .wait();
    }

    HandleResult::Close
}
