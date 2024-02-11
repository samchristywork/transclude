use clap::{arg, Arg, Command};
use std::{
    fs::File,
    io::{Read, Write},
    process,
};

const RED: &str = "\x1b[31m";
const GREY: &str = "\x1b[90m";
const RESET: &str = "\x1b[0m";

fn read_file(filename: &str) -> String {
    eprintln!("{GREY}Reading file:{RESET} {}", filename);
    let mut file = File::open(filename).unwrap_or_else(|_| {
        eprintln!("{RED}File not found:{RESET} {}", filename);
        process::exit(1);
    });
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap_or_else(|_| {
        eprintln!("{RED}Could not read file:{RESET} {}", filename);
        process::exit(1);
    });
    contents
}

fn render(
    filename: &str,
    start_pattern: &str,
    end_pattern: &str,
    file_stack: &mut Vec<String>,
    dot_file: &mut Option<File>,
) -> String {
    if file_stack.contains(&filename.to_string()) {
        file_stack.push(filename.to_string());
        eprintln!("{RED}Cycle detected:{RESET} {:?}", file_stack);
        process::exit(1);
    } else {
        file_stack.push(filename.to_string());
    }

    let file_contents = read_file(filename);
    let mut output = String::new();

    for line in file_contents.lines() {
        if line.contains(start_pattern) {
            let start = line.find(start_pattern).unwrap_or_else(|| {
                eprintln!(
                    "{RED}Could not find start pattern:{RESET} {}",
                    start_pattern
                );
                process::exit(1);
            }) + start_pattern.len();
            let end = line.find(end_pattern).unwrap_or_else(|| {
                eprintln!("{RED}Could not find end pattern:{RESET} {}", end_pattern);
                process::exit(1);
            });
            let include_filename = &line[start..end];

            if let Some(dot_file) = dot_file {
                writeln!(dot_file, "  \"{}\" -> \"{}\";", filename, include_filename).unwrap();
            }

            let include_contents = render(
                include_filename,
                start_pattern,
                end_pattern,
                file_stack,
                dot_file,
            );
            let line_start = &line[..start - start_pattern.len()];
            let line_end = &line[end + end_pattern.len()..];

            output.push_str(line_start);
            output.push_str(&include_contents);
            output.push_str(line_end);
        } else {
            output.push_str(line);
            output.push_str("\n");
        }
    }

    output
}

fn main() {
    let matches = Command::new("transclude")
        .version("0.1.0")
        .author("Sam Christy")
        .about("Transclude files")
        .arg(arg!(-s --start <VALUE> "The start pattern. Default: 'include{'"))
        .arg(arg!(-e --end <VALUE> "The end pattern. Default: '}'"))
        .arg(arg!(-d --dot <FILE> "Write a graphviz dot file"))
        .arg(arg!(-D --dotstyle <STYLE> "The style for the dot file. Default: 'rankdir=LR;node [shape=box];'"))
        .arg(Arg::new("input"))
        .get_matches();

    let binding = String::from("include{");
    let start_pattern = matches.get_one("start").unwrap_or(&binding);

    let binding = String::from("}");
    let end_pattern = matches.get_one("end").unwrap_or(&binding);

    let dot_filename: Option<String> = matches.get_one("dot").map(|s: &String| s.to_string());

    let mut dot_file = match dot_filename {
        Some(filename) => Some(File::create(filename.clone()).unwrap_or_else(|_| {
            eprintln!("{RED}Could not create file:{RESET} {}", filename);
            process::exit(1);
        })),
        None => None,
    };

    let binding = String::from("dotstyle");
    let dot_style = match matches.get_one(&binding).map(|s: &String| s.to_string()) {
        Some(style) => style,
        None => String::from("rankdir=LR;node [shape=box];"),
    };

    match matches.get_one::<String>("input") {
        Some(input_filename) => {
            let file_stack = &mut Vec::new();

            match &mut dot_file {
                Some(dot_file) => {
                    writeln!(dot_file, "digraph G {{\n  {dot_style};").unwrap();
                }
                None => {}
            }

            let contents = render(
                &input_filename,
                start_pattern,
                end_pattern,
                file_stack,
                &mut dot_file,
            );

            match &mut dot_file {
                Some(dot_file) => {
                    writeln!(dot_file, "}}").unwrap();
                }
                None => {}
            }

            println!("{}", contents);
        }
        None => {
            eprintln!("{RED}An input file is required{RESET}");
            process::exit(1);
        }
    }
}
