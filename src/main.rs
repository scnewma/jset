use clap::{App, Arg, ArgMatches, SubCommand};
use std::io;

fn main() -> Result<(), JsetError> {
    let common_args = [
        Arg::with_name("paths")
            .index(1)
            .required(true)
            .min_values(2)
            .multiple(true),
        Arg::with_name("compact")
            .short("c")
            .help("compact instead of pretty-printed output"),
    ];

    let app_matches = App::new("jset")
        .about("a command-line tool for performing set operations on json files")
        .subcommand(
            SubCommand::with_name("intersect")
                .about("calculate the intersection of multiple json files")
                .args(&common_args),
        )
        .subcommand(
            SubCommand::with_name("union")
                .about("calculate the union of multiple json files")
                .args(&common_args),
        )
        .subcommand(
            SubCommand::with_name("difference")
                .visible_alias("diff")
                .about("calculate the difference of multiple json files. files 2..n are subtracted from the first file")
                .args(&common_args),
        )
        .get_matches();

    match app_matches.subcommand() {
        (name, Some(matches)) => {
            let cli = Cli::from(matches);
            match name {
                "intersect" => cli.execute(jset::intersect),
                "union" => cli.execute(jset::union),
                "difference" => cli.execute(jset::difference),
                name => panic!("unknown subcommand {}", name),
            }
        }
        (name, _) => panic!("unknown subcommand {}", name),
    }
}

#[derive(Debug)]
enum JsetError {
    NotFound,
    Generic(String),
}

impl From<io::Error> for JsetError {
    fn from(err: io::Error) -> Self {
        JsetError::Generic(err.to_string())
    }
}

impl From<serde_json::Error> for JsetError {
    fn from(err: serde_json::Error) -> Self {
        JsetError::Generic(err.to_string())
    }
}

struct Cli {
    paths: Vec<String>,
    compact: bool,
}

impl From<&ArgMatches<'_>> for Cli {
    fn from(matches: &ArgMatches) -> Self {
        Cli {
            paths: matches
                .values_of("paths")
                .unwrap()
                .map(|s| s.to_owned())
                .collect(),
            compact: matches.is_present("compact"),
        }
    }
}

impl Cli {
    fn execute<F>(&self, operation: F) -> Result<(), JsetError>
    where
        F: Fn(&serde_json::Value, &serde_json::Value) -> Option<serde_json::Value>,
    {
        let mut paths_iter = self.paths.iter();
        let mut acc = read_to_value(paths_iter.next().unwrap())?;
        for path in paths_iter {
            let v = read_to_value(path)?;
            acc = match operation(&acc, &v) {
                Some(iv) => iv,
                None => return Err(JsetError::NotFound),
            }
        }

        if self.compact {
            println!("{}", serde_json::to_string(&acc)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&acc)?);
        }

        Ok(())
    }
}

fn read_to_value(path: &str) -> Result<serde_json::Value, JsetError> {
    let str = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str::<serde_json::Value>(&str)?)
}
