use sroogle::config::Config;
use std::process;

fn main() {
    /*    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1)
    });
     */
    let config = Config {
        query: String::from("test"),
        directory: String::from("/home/vb/data/docs.gl/gl4"),
        ignore_case: true,
    };
    if let Err(err) = sroogle::run(config) {
        eprintln!("Application error: {}", err);
        process::exit(1)
    }
}
