use std::env;

pub struct Config {
    pub query: String,
    pub directory: String,
    pub ignore_case: bool,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, String> {
        let _ = args.next();

        let ignore_case;
        let query;
        let arg_1 = args.next().ok_or("Didn't get a query string".to_string())?;
        if arg_1 == "-i" || arg_1 == "--ignore-case" {
            ignore_case = true;
            query = args.next().ok_or("Didn't get a query string".to_string())?;
        } else {
            ignore_case = env::var("IGNORE_CASE").is_ok();
            query = arg_1;
        }

        let directory = args.next().ok_or("Didn't get a directory".to_string())?;
        if let Some(unexpected) = args.next() {
            Err(format!("Unexpected argument: {}", unexpected))
        } else {
            Ok(Config {
                query,
                directory,
                ignore_case,
            })
        }
    }
}
