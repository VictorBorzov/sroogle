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
use tiny_http::{Header, Response, Server};
use urlencoding::*;
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
            *tf.entry(token).or_insert(0) += 1;
        }
        if !tf.is_empty() {
            index.insert(file_path, tf);
        } else {
            eprintln!(
                "WARNING: nothing indexed in {file_path}",
                file_path = file_path.as_path().display()
            );
        }
    }

    Ok(())
}

/// Parse url parameters to hashmap like /?query=test+query -> { query : test query }
/// Example:
/// ```
/// assert_eq!(get_param_value_from_url("/?query=test+query"), Some("test query".to_string()))
/// ```
fn get_param_value_from_url(url: &str, param_name: &str) -> Option<String> {
    let start: usize = url.match_indices(param_name).map(|p| p.0).next()?;
    let start_value: usize = url
        .match_indices('=')
        .filter(|p| p.0 > start)
        .map(|p| p.0)
        .next()?
        + 1;
    match url
        .match_indices('&')
        .filter(|p| p.0 > start_value)
        .map(|p| p.0)
        .next()
    {
        None => Some(url[start_value..].replace('+', " ")),
        Some(i) => Some(url[start_value..i].replace('+', " ")),
    }
}

fn calc_tf(tf: &TermFreq, term: &str) -> f32 {
    let m = *tf.get(term).unwrap_or(&0usize) as f32;
    let n = tf.values().sum::<usize>() as f32;
    m / n
}

fn calc_idf(tf_index: &TermFreqIndex, term: &str) -> f32 {
    let n = tf_index.len() as f32;
    let d = tf_index
        .iter()
        .filter(|(_, tf)| tf.contains_key(term))
        .count() as f32;
    (n / d.max(1f32)).log10()
}

fn calc_rate<'a>(tf_index: &'a TermFreqIndex, term: &str) -> HashMap<&'a PathBuf, f32> {
    let idf = calc_idf(&tf_index, term);
    tf_index
        .iter()
        .map(|(p, tf)| (p, calc_tf(&tf, term) * idf))
        .collect()
}

pub fn index_directory(directory: String) -> Result<(), Box<dyn Error>> {
    let mut tf_index = TermFreqIndex::new();
    index_dir(Path::new(&directory), &mut tf_index)?;
    Ok(save_tf_index(&tf_index, "index.json")?)
}

//todo: use build_html
fn serve(index_path: &str, address: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let server = Server::http(address)?;

    println!("INFO: listening at http://{address}/");
    let file = File::open(index_path)?;
    let tf_index: TermFreqIndex = serde_json::from_reader(file)?;

    for request in server.incoming_requests() {
        println!(
            "INFO: received request! method: {:?}, url: {:?}",
            request.method(),
            request.url(),
        );

        let mut html_response = String::from(
            r#"
          <html>
            <header>
              <title>Sroogle</title>
            </header>
            <body>
              <form action="/" method="get">
                <h1>Query</h1>
                <input id="query" type="text" name="query"/>
                <input type="submit" value="Submit">
              </form>
        "#,
        );

        if let Some(query) = get_param_value_from_url(decode(&request.url())?.as_ref(), "query") {
            html_response.push_str("<h1>Results</h1>");
            html_response.push_str("<ul>");
            let chars: Vec<_> = dbg!(query).chars().collect();
            let lexer = Lexer::build(&chars);
            let mut rates: HashMap<&PathBuf, HashMap<String, f32>> = HashMap::new();
            for token in lexer {
                for (path, rate) in calc_rate(&tf_index, &token) {
                    rates
                        .entry(path)
                        .or_insert(HashMap::new())
                        .insert(token.clone(), rate);
                }
            }
            let mut sorted: Vec<(&PathBuf, HashMap<String, f32>)> = rates.into_iter().collect();
            sorted.sort_by(|(_, lhs_r), (_, rhs_r)| {
                rhs_r
                    .values()
                    .sum::<f32>()
                    .partial_cmp(&lhs_r.values().sum::<f32>())
                    .unwrap()
            });
            for (path, rates) in sorted.iter().take(5) {
                html_response.push_str(&format!("<li>{path:?}:</li>"));
                html_response.push_str("<ul>");
                for (token, rate) in rates {
                    html_response.push_str(&format!("<li>    {token}: {rate}</li>"));
                }
                html_response.push_str("</ul>");
            }
            html_response.push_str("</ul>");
        }
        html_response.push_str(
            r#"
              </body>
          </html>"#,
        );

        let content_type_text_html = Header::from_bytes("Content-Type", "text/html")
            .expect("That we didn't put any garbage in the headers");

        let response = Response::from_string(html_response).with_header(content_type_text_html);
        request
            .respond(response)
            .unwrap_or_else(|err| eprintln!("ERROR: could not serve a request: {err}"));
    }
    Ok(())
}

fn parse_args_and_run(args: impl Iterator<Item = String>) -> Result<(), String> {
    match Command::build(args) {
        Some(Command::IndexDirectory(directory)) => {
            index_directory(directory).map_err(|e| format!("{e}"))
        }
        Some(Command::Search(query)) => query_index(&query).map_err(|e| format!("{e}")),
        Some(Command::Serve {
            index_path,
            address,
        }) => serve(&index_path, &address).map_err(|e| format!("{e}")),
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_url_parsed() {
        assert_eq!(
            get_param_value_from_url("/?query=test+query", "query"),
            Some("test query".to_string())
        )
    }
}
