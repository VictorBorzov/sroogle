pub enum Command {
    IndexDirectory(String),
    Search(String),
}

impl Command {
    pub fn get_subcommands_description() -> &'static str {
        "
    index <folder>          index the <folder> and save the index to index.json file
    search <index-file>    check how many documents are indexed in the file (searching is not implemented yet)"
    }

    pub fn build(mut args: impl Iterator<Item = String>) -> Option<Command> {
        match args.next()?.to_lowercase().as_str() {
            "index" => Some(Command::IndexDirectory(args.next()?)),
            "search" => Some(Command::Search(args.next()?)),
            _ => None,
        }
    }
}
