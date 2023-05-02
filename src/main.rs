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
    let modifiers = vec![
        "Shift_L",
        "Shift_R",
        "Control_L",
        "Control_R",
        "Alt_L",
        "Alt_R",
    ];

    for path in paths {
        let mut pressed: HashMap<String, bool> = modifiers
            .iter()
            .map(|key| (key.to_string(), false))
            .collect();

        let data = std::fs::read_to_string(&path)
            .with_context(|| format!("Could not read file `{}`", path.display()))?;
        for line in data.split("\n") {
            let split = line.split_whitespace().collect::<Vec<&str>>();
            if split.len() != 2 {
                continue;
            }
            let keycode = split[0];
            let action = split[1];
            let modifier = &keycodes[keycode];
            if modifiers.iter().any(|key| key == &modifier) {
                match action {
                    "(KeyPress)" => pressed.insert(modifier.clone(), true),
                    "(KeyRelease)" => pressed.insert(modifier.clone(), false),
                    _ => return Err(anyhow::anyhow!("Unknown action `{}`", action)),
                };
            }

            let mut letter: String = match keycodes.get(keycode) {
                Some(letter) => letter.clone(),
                None => continue,
            };

            if pressed["Shift_L"] || pressed["Shift_R"] {
                letter = match letter.as_str() {
                    "1" => String::from("!"),
                    "2" => String::from("@"),
                    "3" => String::from("#"),
                    "4" => String::from("$"),
                    "5" => String::from("%"),
                    "6" => String::from("^"),
                    "7" => String::from("&"),
                    "8" => String::from("*"),
                    "9" => String::from("("),
                    "0" => String::from(")"),
                    "minus" => String::from("_"),
                    "equal" => String::from("+"),
                    "bracketright" => String::from("{"),
                    "bracketleft" => String::from("}"),
                    "semicolon" => String::from(":"),
                    "apostrophe" => String::from("\""),
                    "backslash" => String::from("|"),
                    "comma" => String::from("<"),
                    "period" => String::from(">"),
                    "slash" => String::from("?"),
                    "grave" => String::from("~"),
                    _ => letter,
                };
            } else {
                letter = match letter.as_str() {
                    "minus" => String::from("-"),
                    "equal" => String::from("="),
                    "bracketright" => String::from("]"),
                    "bracketleft" => String::from("["),
                    "semicolon" => String::from(";"),
                    "apostrophe" => String::from("'"),
                    "backslash" => String::from("\\"),
                    "comma" => String::from(","),
                    "period" => String::from("."),
                    "slash" => String::from("/"),
                    "grave" => String::from("`"),
                    _ => letter,
                }
            }

            let count = histogram.entry(letter.clone()).or_insert(0 as u32);
            *count += 1;
        }
    }

    return Ok(histogram);
}
