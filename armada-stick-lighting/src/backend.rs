//! Hardware backends for stick lighting.

use crate::LightingConfig;
use anyhow::{bail, Context, Result};
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub enum LightingBackend {
    Multicolor(MulticolorBackend),
    Unsupported(String),
}

impl LightingBackend {
    pub fn apply(&self, config: &LightingConfig) -> Result<()> {
        match self {
            Self::Multicolor(backend) => backend.apply(config),
            Self::Unsupported(reason) => bail!("{reason}"),
        }
    }

    pub fn unsupported_reason(&self) -> Option<&str> {
        match self {
            Self::Multicolor(_) => None,
            Self::Unsupported(reason) => Some(reason),
        }
    }
}

pub struct MulticolorBackend {
    root: PathBuf,
    targets: Vec<String>,
}

impl MulticolorBackend {
    pub fn new(root: PathBuf, targets: Vec<String>) -> Self {
        Self { root, targets }
    }

    fn apply(&self, config: &LightingConfig) -> Result<()> {
        let mut targets: Vec<PreparedTarget> = self.prepare(config)?;

        if !config.enabled {
            return blank(&mut targets);
        }

        if let Err(error) = blank(&mut targets) {
            blank_best_effort(&mut targets);
            return Err(error);
        }
        if let Err(error) = write_colors(&mut targets) {
            blank_best_effort(&mut targets);
            return Err(error);
        }
        if let Err(error) = write_brightness(&mut targets) {
            blank_best_effort(&mut targets);
            return Err(error);
        }
        Ok(())
    }

    fn prepare(&self, config: &LightingConfig) -> Result<Vec<PreparedTarget>> {
        validate_names(&self.targets)?;
        let mut targets: Vec<PreparedTarget> = Vec::new();
        let rgb: [u8; 3] = config.rgb();

        for name in &self.targets {
            let path: PathBuf = self.root.join(name);
            let brightness_path: PathBuf = path.join("brightness");
            let blank: File = OpenOptions::new()
                .write(true)
                .open(&brightness_path)
                .with_context(|| format!("open {name} brightness"))?;

            if !config.enabled {
                targets.push(PreparedTarget {
                    name: name.clone(),
                    brightness_path,
                    blank,
                    brightness: None,
                    color: None,
                });
                continue;
            }

            let order: Vec<String> = read_order(&path.join("multi_index"))?;
            let maximum: u32 = read_maximum(&path.join("max_brightness"))?;
            let values: Vec<String> = order
                .iter()
                .map(|channel| channel_value(channel, rgb, maximum).to_string())
                .collect();
            let intensity: File = OpenOptions::new()
                .write(true)
                .open(path.join("multi_intensity"))
                .with_context(|| format!("open {name} multi_intensity"))?;
            let brightness: File = OpenOptions::new()
                .write(true)
                .open(&brightness_path)
                .with_context(|| format!("open {name} brightness"))?;
            let brightness_value: String = scale(config.brightness, maximum).to_string();

            targets.push(PreparedTarget {
                name: name.clone(),
                brightness_path,
                blank,
                brightness: Some((brightness, brightness_value)),
                color: Some((intensity, values.join(" "))),
            });
        }
        Ok(targets)
    }
}

struct PreparedTarget {
    name: String,
    brightness_path: PathBuf,
    blank: File,
    brightness: Option<(File, String)>,
    color: Option<(File, String)>,
}

fn blank(targets: &mut [PreparedTarget]) -> Result<()> {
    for target in targets {
        write_attr(&mut target.blank, "0")
            .with_context(|| format!("write {} brightness", target.name))?;
    }
    Ok(())
}

fn blank_best_effort(targets: &mut [PreparedTarget]) {
    for target in targets {
        let _ = fs::write(&target.brightness_path, b"0\n");
    }
}

fn write_colors(targets: &mut [PreparedTarget]) -> Result<()> {
    for target in targets {
        let (file, value) = target.color.as_mut().expect("prepared color");
        write_attr(file, value).with_context(|| format!("write {} color", target.name))?;
    }
    Ok(())
}

fn write_brightness(targets: &mut [PreparedTarget]) -> Result<()> {
    for target in targets {
        let (file, value) = target.brightness.as_mut().expect("prepared brightness");
        write_attr(file, value).with_context(|| format!("write {} brightness", target.name))?;
    }
    Ok(())
}

fn write_attr(file: &mut File, value: &str) -> std::io::Result<()> {
    let output: String = format!("{value}\n");
    file.write_all(output.as_bytes())?;
    file.flush()
}

fn read_order(path: &Path) -> Result<Vec<String>> {
    let input: String =
        fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let order: Vec<String> = input.split_whitespace().map(str::to_lowercase).collect();
    let channels: HashSet<&str> = order.iter().map(String::as_str).collect();

    if order.len() != 3 || channels != HashSet::from(["red", "green", "blue"]) {
        bail!(
            "{} is not an RGB multi_index: '{}'",
            path.display(),
            input.trim()
        );
    }
    Ok(order)
}

fn read_maximum(path: &Path) -> Result<u32> {
    let input: String =
        fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let maximum: u32 = input
        .trim()
        .parse()
        .with_context(|| format!("parse {}", path.display()))?;

    if maximum == 0 {
        bail!("{} is zero", path.display());
    }
    Ok(maximum)
}

fn validate_names(targets: &[String]) -> Result<()> {
    let mut seen: HashSet<&String> = HashSet::new();

    if targets.is_empty() {
        bail!("stick-lighting target list is empty");
    }
    for target in targets {
        let valid: bool = !target.is_empty()
            && target
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b":_.-".contains(&c))
            && target != "."
            && target != "..";
        if !valid {
            bail!("invalid target name '{target}'");
        }
        if !seen.insert(target) {
            bail!("duplicate target '{target}'");
        }
    }
    Ok(())
}

fn channel_value(channel: &str, [red, green, blue]: [u8; 3], maximum: u32) -> u32 {
    match channel {
        "red" => gamma(red, maximum),
        "green" => gamma(green, maximum),
        "blue" => gamma(blue, maximum),
        _ => unreachable!("validated channel"),
    }
}

fn gamma(channel: u8, maximum: u32) -> u32 {
    let value: f64 = f64::from(channel) / 255.0;
    let linear: f64 = if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    };
    (linear * f64::from(maximum)).round() as u32
}

fn scale(percent: u8, maximum: u32) -> u32 {
    ((u64::from(percent) * u64::from(maximum) + 50) / 100) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scales_channels_and_brightness() {
        assert_eq!(gamma(0, 255), 0);
        assert_eq!(gamma(128, 100), 22);
        assert_eq!(gamma(255, 255), 255);
        assert_eq!(scale(25, 255), 64);
    }
}
