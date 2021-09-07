use clap::{App, Arg};

#[derive(Debug)]
pub struct Config {
    pub recompile:  bool,
    pub repos:      Vec<String>,
    pub mod_folder: String
}

impl Config {
    pub fn new() -> Self {
        Self::get_args(Self::matches())
    }

    fn get_args(matches: clap::ArgMatches<'_>) -> Self {
        let recompile = matches.is_present("recompile");
        let mod_folder = matches.value_of("mod-folder").unwrap().to_string(); // Always Some due to a default value
        let repos: Vec<String> = if let Some(r) = matches.values_of("repo") {
            r.map(|f| f.to_string()).collect()
        } else {
            vec![]
        };

        Config {
            recompile,
            repos,
            mod_folder
        }
    }

    fn matches<'a>() -> clap::ArgMatches<'a> {
        App::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .arg(Arg::with_name("repo")
                .short("r")
                .long("repo")
                .takes_value(true)
                .multiple(true)
                .required(false)
                .value_name("Repository URL"))
            .arg(Arg::with_name("recompile")
                .long("recompile")
                .takes_value(false)
                .required(false))
            .arg(Arg::with_name("mod-folder")
                .short("m")
                .long("mod-folder")
                .takes_value(true)
                .required(true)
                .default_value("/modules"))
            .get_matches()
    }
}

