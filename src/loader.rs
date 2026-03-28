use core::f64;
use std::{env, fs, path::PathBuf};

use strsim::jaro;
use walkdir::WalkDir;

fn normalize(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_space = false;

    for ch in input.chars().flat_map(|c| c.to_lowercase()) {
        if ch.is_alphanumeric() {
            out.push(ch);
            last_space = false;
        } else {
            if !last_space && out.is_empty() {
                out.push(' ');
                last_space = true;
            }
        }
    }

    out.trim().to_string()
}

fn split_artist_title(input: &str) -> Option<(&str, &str)> {
    if let Some((a, t)) = input.split_once('-') {
        let a = a.trim();
        let t = t.trim();
        if !a.is_empty() && !t.is_empty() {
            return Some((a, t));
        }
    }
    None
}

fn score(filename: &str, norm_artist: &str, norm_title: &str) -> f64 {
    let norm_name = normalize(filename);
    if let Some((a, t)) = split_artist_title(filename) {
        let norm_a = normalize(a);
        let norm_t = normalize(t);

        let mut score = 0.0;

        if norm_t == norm_title {
            score += 1.0;
        }
        if norm_a == norm_artist {
            score += 1.0;
        }

        let a_score = jaro(&norm_a, norm_artist);
        let t_score = jaro(&norm_t, norm_artist);

        score + a_score * 0.4 + t_score * 0.6
    } else {
        let artist_score = jaro(&norm_name, norm_artist);
        let title_score = jaro(&norm_name, norm_title);

        artist_score * 0.3 + title_score * 0.7
    }
}

pub fn load_lyrics(artist: &str, title: &str) -> Option<String> {
    let directory_path = PathBuf::from(match env::var("LYRICS_DIR") {
        Ok(path) => path,
        Err(_) => format!("{}/.local/share/lyrics", env::var("HOME").unwrap_or("/root".to_string()))
    });
    let mut best: Option<(PathBuf, f64)> = None;

    let norm_artist = &normalize(artist);
    let norm_title = &normalize(title);
    let recursive = WalkDir::new(directory_path);
    for entry in recursive
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s,
            None => continue
        };

        let score = score(stem, norm_artist, norm_title);
        if score < 0.5 {
            continue;
        }
        match &best {
            Some((_, best_score)) if score <= *best_score => {},
            _ => best = Some((path.to_path_buf(), score))
        }
    }

    best.and_then(|(p, _)| {
        fs::read_to_string(p).ok()
    })
}

