use anyhow::{Context, Result};
use clap::Parser;
use std::{collections::HashMap, process::Command};

#[derive(Parser, Debug)]
struct Cli {
    #[arg(num_args(0..))]
    path: Option<Vec<std::path::PathBuf>>,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let keycodes = get_keycodes().with_context(|| "Could not get keycodes")?;

    let paths = get_paths(args.path).with_context(|| "Could not get paths")?;

    let histogram =
        calculate_histogram(paths, keycodes).with_context(|| "Could not calculate histogram")?;

    let mut counts: Vec<_> = histogram.iter().collect();
    counts.sort_by(|a, b| a.1.cmp(&b.1));
    for (letter, count) in counts {
        println!("{}: {}", letter, count);
    }
    return Ok(());
}

fn get_keycodes() -> Result<HashMap<String, String>> {
    let output = Command::new("xmodmap")
        .arg("-pke")
        .output()
        .expect("failed to execute process");

    let output = String::from_utf8(output.stdout).expect("Could not parse output");
    let output = output.split("\n").collect::<Vec<&str>>();

    let mut keycodes = HashMap::new();

    for line in output {
        let line = line.split(" = ").collect::<Vec<&str>>();
        if line.len() == 2 {
            let keycode = line[0].split_whitespace().collect::<Vec<&str>>()[1];

            let key = line[1].split(" ").collect::<Vec<&str>>()[0];

            keycodes.insert(keycode.to_string(), key.to_string());
        }
    }
    return Ok(keycodes);
}

fn get_paths(arg_paths: Option<Vec<std::path::PathBuf>>) -> Result<Vec<std::path::PathBuf>> {
    let mut paths = match arg_paths {
        Some(paths) => paths,
        None => vec![std::path::PathBuf::from("/dev/stdin")],
    };

    let mut dirs = vec![];

    paths.retain(|path| {
        if path.is_dir() {
            dirs.push(path.clone());
            return false;
        } else {
            return true;
        }
    });

    for path in dirs {
        let mut files = std::fs::read_dir(&path)
            .with_context(|| format!("Could not read directory `{}`", path.display()))?;
        while let Some(file) = files.next() {
            let file = file?;
            let path = file.path();
            if path.is_file() {
                paths.push(path);
            }
        }
    }
    return Ok(paths);
}

fn calculate_histogram(
    paths: Vec<std::path::PathBuf>,
    keycodes: HashMap<String, String>,
) -> Result<HashMap<String, u32>> {
    let mut histogram = HashMap::new();

    for path in paths {
        let data = std::fs::read_to_string(&path)
            .with_context(|| format!("Could not read file `{}`", path.display()))?;
        for keycode in data.split("\n") {
            let letter = match keycodes.get(keycode) {
                Some(letter) => letter,
                None => continue,
            };

            let count = histogram.entry(letter.clone()).or_insert(0 as u32);
            *count += 1;
        }
    }

    return Ok(histogram);
}
