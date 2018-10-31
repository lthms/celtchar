extern crate clap;

use clap::{App, SubCommand};

fn main() {
    let matches = App::new("celtchar")
        .version("0.1")
        .author("Thomas Letan")
        .about("A tool to generate novels")
        .subcommand(SubCommand::with_name("new")
                    .about("Create a new celtchar document"))
        .get_matches();

    let (subcommand, _args) = matches.subcommand();

    match subcommand {
        "new" => println!("Hello, world!"),
        _     => println!("meh"),
    }
}
