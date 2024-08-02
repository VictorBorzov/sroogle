pub enum Command {
    IndexDirectory(String),
    Search(String),
    Serve { index_path: String, address: String },
}

impl Command {
    pub fn get_subcommands_description() -> &'static str {
        "
    index <folder>                       index the <folder> and save the index to index.json file
    search <index-file>                  check how many documents are indexed in the file (searching is not implemented yet)
    serve <index-file> [address]         start local HTTP server with Web Interface"
    }

    pub fn build(mut args: impl Iterator<Item = String>) -> Option<Command> {
        match args.next()?.to_lowercase().as_str() {
            "index" => Some(Command::IndexDirectory(args.next()?)),
            "search" => Some(Command::Search(args.next()?)),
            "serve" => Some(Command::Serve {
                index_path: args.next()?,
                address: args.next().unwrap_or("127.0.0.1:6969".to_string()),
            }),
            _ => None,
        }
    }
}
