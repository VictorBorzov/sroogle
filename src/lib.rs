pub mod config;
mod lexer;
use config::Config;
use lexer::Lexer;
use serde_json::json;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Result;
use std::path::{Path, PathBuf};
use xml::reader::{EventReader, XmlEvent};

#[allow(dead_code)]
fn index_document(_doc_content: &str) -> HashMap<String, usize> {
    todo!("not implemented");
}

type TermFreq = HashMap<String, usize>;
type TermFreqIndex = HashMap<PathBuf, TermFreq>;

pub fn run(config: Config) -> Result<()> {
    let index_path = "index.json";
    if let Ok(file) = File::open(index_path) {
        let tf_index: TermFreqIndex = serde_json::from_reader(file)?;
        for (path, tf) in tf_index {
            println!("{path:?} has {number} unique tokens", number = tf.len());
        }
    } else {
        let mut tf_index = TermFreqIndex::new();

        for file in fs::read_dir(config.directory)? {
            let file_path = file?.path();

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

            let mut stats = tf.iter().collect::<Vec<_>>();
            stats.sort_by_key(|k| k.1);
            stats.reverse();

            tf_index.insert(file_path, tf);
        }

        println!("Saving {index_path}...");
        let file_buffer = File::create("index.json")?;
        serde_json::to_writer_pretty(file_buffer, &json!(tf_index))?;
    }
    Ok(())
}

fn read_text_from_xml<P: AsRef<Path>>(file: P) -> Result<String> {
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
