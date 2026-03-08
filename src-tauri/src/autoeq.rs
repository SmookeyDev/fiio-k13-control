use serde::{Deserialize, Serialize};
use crate::error::{AppError, Result};

const INDEX_URL: &str =
    "https://raw.githubusercontent.com/jaakkopasanen/AutoEq/master/results/INDEX.md";
const RAW_BASE: &str =
    "https://raw.githubusercontent.com/jaakkopasanen/AutoEq/master/results/";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoEqHeadphone {
    pub name: String,
    pub path: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoEqProfile {
    pub preamp: f64,
    pub filters: Vec<AutoEqFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoEqFilter {
    pub index: u8,
    pub enabled: bool,
    pub filter_type: String,
    pub frequency: f64,
    pub gain: f64,
    pub q: f64,
}

pub fn fetch_index() -> Result<Vec<AutoEqHeadphone>> {
    let resp = reqwest::blocking::get(INDEX_URL)?;
    let text = resp.text()?;

    let mut headphones = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with("- [") {
            continue;
        }

        let name_end = match line.find("](") {
            Some(i) => i,
            None => continue,
        };
        let name = line[3..name_end].to_string();

        let path_start = name_end + 2;
        let path_end = {
            let sub = &line[path_start..];
            let mut depth = 0i32;
            let mut end = None;
            for (i, c) in sub.char_indices() {
                match c {
                    '(' => depth += 1,
                    ')' => {
                        if depth == 0 { end = Some(path_start + i); break; }
                        depth -= 1;
                    }
                    _ => {}
                }
            }
            match end { Some(e) => e, None => continue }
        };
        let raw_path = &line[path_start..path_end];
        let path = raw_path.strip_prefix("./").unwrap_or(raw_path).to_string();

        let after_paren = &line[path_end + 1..];
        let source = if let Some(rest) = after_paren.strip_prefix(" by ") {
            rest.split(" on ").next().unwrap_or("").trim().to_string()
        } else {
            String::new()
        };

        headphones.push(AutoEqHeadphone { name, path, source });
    }

    Ok(headphones)
}

pub fn fetch_parametric_eq(path: &str) -> Result<AutoEqProfile> {
    let model_name = path.rsplit('/').next().unwrap_or(path);
    let url = format!("{}{}/{}%20ParametricEQ.txt", RAW_BASE, path, model_name);

    let resp = reqwest::blocking::get(&url)?;

    if !resp.status().is_success() {
        return Err(AppError::Protocol(format!(
            "Parametric EQ file not found (HTTP {})", resp.status()
        )));
    }

    let text = resp.text()?;
    parse_parametric_eq(&text)
}

fn parse_parametric_eq(text: &str) -> Result<AutoEqProfile> {
    let mut preamp = 0.0;
    let mut filters = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("Preamp:") {
            if let Some(val) = line
                .strip_prefix("Preamp:")
                .and_then(|s| s.trim().strip_suffix("dB"))
                .and_then(|s| s.trim().parse::<f64>().ok())
            {
                preamp = val;
            }
        } else if line.starts_with("Filter") {
            parse_filter_line(line, &mut filters);
        }
    }

    Ok(AutoEqProfile { preamp, filters })
}

fn parse_filter_line(line: &str, filters: &mut Vec<AutoEqFilter>) {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 12 {
        return;
    }

    let index = parts[1].trim_end_matches(':').parse::<u8>().unwrap_or(0);
    let enabled = parts[2] == "ON";
    let filter_type = parts[3].to_string();
    let frequency = parts[5].parse::<f64>().unwrap_or(1000.0).clamp(20.0, 20000.0);
    let gain = parts[8].parse::<f64>().unwrap_or(0.0).clamp(-12.0, 12.0);
    let q = parts[11].parse::<f64>().unwrap_or(1.0).clamp(0.1, 10.0);

    filters.push(AutoEqFilter {
        index: index.saturating_sub(1),
        enabled,
        filter_type,
        frequency,
        gain,
        q,
    });
}
