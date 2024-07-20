mod command;
mod lexer;
use command::Command;
use lexer::Lexer;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use xml::reader::{EventReader, XmlEvent};

fn read_text_from_xml<P: AsRef<Path>>(file: P) -> Result<String, Box<dyn std::error::Error>> {
    let file = File::open(file)?;
    let er = EventReader::new(file);
    let mut content = String::new();
    for event in er.into_iter().flatten() {
        if let XmlEvent::Characters(el) = event {
            content.push_str(&el);
            content.push(' ');
        }
    }
    Ok(content)
}

type TermFreq = HashMap<String, usize>;
type TermFreqIndex = HashMap<PathBuf, TermFreq>;

fn query_index(index_path: &str) -> std::io::Result<()> {
    println!("Using index file {index_path}");
    let file = File::open(index_path)?;
    let tf_index: TermFreqIndex = serde_json::from_reader(file)?;
    println!("Index contains {l} files", l = tf_index.len());
    Ok(())
}

fn save_tf_index(index: &TermFreqIndex, index_path: &str) -> std::io::Result<()> {
    println!("Saving {index_path}...");
    let file_buffer = File::create(index_path)?;
    serde_json::to_writer_pretty(file_buffer, &json!(index))?;
    Ok(())
}

fn index_dir(directory: &Path, index: &mut TermFreqIndex) -> Result<(), Box<dyn Error>> {
    let dir_iter = fs::read_dir(directory)?;
    for file in dir_iter {
        let file = file?;
        let file_path = file.path();
        if file.file_type()?.is_dir() {
            index_dir(&file_path, index)?;
            continue;
        }

        println!("Indexing {:?}...", &file_path);

        let content = read_text_from_xml(&file_path)?.chars().collect::<Vec<_>>();

        let mut tf = TermFreq::new();

        let lexer = Lexer::build(&content);
        for token in lexer {
            let term = token
                .iter()
                .map(|c| c.to_ascii_uppercase())
                .collect::<String>();

            *tf.entry(term).or_insert(0) += 1;
        }
        if !tf.is_empty() {
            index.insert(file_path, tf);
        }
    }
    Ok(())
}

pub fn index_directory(directory: String) -> Result<(), Box<dyn Error>> {
    let mut tf_index = TermFreqIndex::new();
    index_dir(Path::new(&directory), &mut tf_index)?;
    Ok(save_tf_index(&tf_index, "index.json")?)
}

fn parse_args_and_run(args: impl Iterator<Item = String>) -> Result<(), String> {
    match Command::build(args) {
        Some(Command::IndexDirectory(directory)) => {
            index_directory(directory).map_err(|e| format!("{e}"))
        }
        Some(Command::Search(query)) => query_index(&query).map_err(|e| format!("{e}")),
        None => Err("Wrong arguments".to_string()),
    }
}

fn main() -> ExitCode {
    let mut args = env::args();
    let app_name = args.next().unwrap_or("NAME NOT FOUND".to_string());

    match parse_args_and_run(args) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!(
                "Usage: {app_name} [SUBCOMMAND] [OPTIONS]\nSubcommands: {desc}\nERROR: {e}",
                desc = Command::get_subcommands_description(),
            );
            ExitCode::FAILURE
        }
    }
}
