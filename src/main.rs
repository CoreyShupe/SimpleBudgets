#[macro_use] extern crate prettytable;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use clap::{App, AppSettings, Arg, SubCommand, ArgMatches};

use crate::budget::{Budget, BudgetPart};

mod budget;

const MAIN_AUTHOR: &str = "CoreyShupe";
const CMD_OPEN_VERSION: &str = "0.0.1";
const CMD_NEW_VERSION: &str = "0.0.1";

fn main() {
    let base_app_matcher = App::new("Simple Budgets")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::ColorAlways)
        .setting(AppSettings::ColorAuto)
        .setting(AppSettings::ColoredHelp)
        .arg(Arg::with_name("file")
            .short("f")
            .required(true)
            .help("Sets the file to interact with.")
            .index(1))
        .subcommand(SubCommand::with_name("open")
            .alias("o")
            .about("Opens a budget file for modification or viewing.")
            .version(CMD_OPEN_VERSION)
            .author(MAIN_AUTHOR))
        .subcommand(SubCommand::with_name("new")
            .alias("n")
            .about("Starts a new budget.")
            .version(CMD_NEW_VERSION)
            .author(MAIN_AUTHOR))
        .author(MAIN_AUTHOR)
        .get_matches();

    let file = Path::new(base_app_matcher.value_of("file").unwrap());

    match base_app_matcher.subcommand_name() {
        Some("open") => {
            if !file.exists() {
                println!("That file does not exist.");
                return;
            }

            let mut string = String::new();
            read_from_file(&file.to_path_buf(), &mut string);

            recursive_interactions(file, string.parse::<Budget>().expect("Failed to read input file as a budget."));
        }
        Some("new") => recursive_interactions(file, Budget::default()),
        _ => unreachable!(),
    }
}

fn recursive_interactions(file: &Path, budget_in: Budget) {
    let recursive_app = App::new("Simple Budgets")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::ColorAlways)
        .setting(AppSettings::ColorAuto)
        .setting(AppSettings::ColoredHelp)
        .subcommand(SubCommand::with_name("exit")
            .alias("e")
            .about("Exits the program in a clean way.")
            .version("0.0.1")
            .author(MAIN_AUTHOR))
        .subcommand(SubCommand::with_name("preview")
            .alias("p")
            .about("Previews the current budget.")
            .version("0.0.1")
            .author(MAIN_AUTHOR))
        .subcommand(SubCommand::with_name("insert")
            .alias("i")
            .about("Inserts a new piece of the budget")
            .version("0.0.1")
            .author(MAIN_AUTHOR)
            .arg(Arg::with_name("name")
                .short("n")
                .help("The name of the part of the budget.")
                .required(true)
                .index(1))
            .arg(Arg::with_name("value")
                .short("v")
                .help("The value of the budget part. The amount going in annually.")
                .required(true)
                .validator(is_number_value)
                .index(2))
            .arg(Arg::with_name("expandable")
                .short("e")
                .help("If the budget part can be mutable. If the part is not required.")
                .required(true)
                .validator(is_bool_value)
                .index(3)))
        .subcommand(SubCommand::with_name("calculate")
            .alias("c")
            .about("Calculates the divided values among the budget parts.")
            .version("0.0.1")
            .author(MAIN_AUTHOR)
            .arg(Arg::with_name("income")
                .short("i")
                .help("The provided income.")
                .required(true)
                .validator(is_number_value)
                .index(1)));

    let mut budget = budget_in;

    let mut exit_flag = false;

    while !exit_flag {
        print!("Please enter a command: ");
        std::io::stdout().flush().expect("Failed to flush stdout.");

        let mut command = String::new();
        std::io::stdin().read_line(&mut command).expect("Failed to gather commandline input.");

        let result = shellwords::split(&command);

        match result {
            Ok(mut vec) => {
                vec.insert(0, String::from("run"));
                println!();
                let x = parse_app_matching(recursive_app.clone().get_matches_from(vec), budget);
                println!();
                exit_flag = x.0;
                budget = x.1;
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    println!("Exiting and saving budget file...");
    write_to_file(&file.to_path_buf(), Into::<String>::into(budget).as_bytes());
    println!("Budget file saved.");
}

fn parse_app_matching(matches: ArgMatches, mut budget: Budget) -> (bool, Budget) {
    match matches.subcommand_name() {
        Some("exit") => (true, budget),
        Some("preview") => {
            budget.print_budget();
            (false, budget)
        }
        Some("insert") => {
            let insert_matches = matches.subcommand_matches("insert")
                .expect("Something bad went wrong...");

            let name = insert_matches.value_of("name")
                .expect("Failed to retrieve required argument.");

            let value_string = insert_matches.value_of("value")
                .expect("Failed to retrieve required argument.");
            let value = from_year_to_months(value_string.parse::<f64>()
                .expect("Failed to validate value arg."));

            let expandable_string = insert_matches.value_of("expandable")
                .expect("Failed to retrieve required argument.");
            let expandable = expandable_string.parse::<bool>()
                .expect("Failed to validate expandable arg.");

            let part = BudgetPart::new(String::from(name), value, expandable);
            println!("Pushed budget: {}, use preview to view the new budget.", part.name());

            budget.push_part(BudgetPart::new(String::from(name), value, expandable));

            (false, budget)
        }
        Some("calculate") => {
            let income_string = matches.subcommand_matches("calculate")
                .expect("Something bad went wrong...")
                .value_of("income")
                .expect("Failed to retrieve required argument.");

            let value = income_string.parse::<f64>()
                .expect("Failed to validate income arg.");

            budget.print_budget_requirement(value);

            (false, budget)
        }
        _ => unreachable!(),
    }
}

fn is_number_value(v: String) -> Result<(), String> {
    if let Ok(_) = v.parse::<f64>() {
        Ok(())
    } else {
        Err(String::from("The provided value is not a valid number."))
    }
}

fn is_bool_value(v: String) -> Result<(), String> {
    if let Ok(_) = v.parse::<bool>() {
        Ok(())
    } else {
        Err(String::from("The provided value is not a boolean value."))
    }
}

fn read_from_file(path: &PathBuf, str: &mut String) {
    OpenOptions::new()
        .read(true)
        .open(path)
        .expect("Failed to open read file.")
        .read_to_string(str)
        .expect("Failed to read string from path.");
}

fn write_to_file(path: &PathBuf, bytes: &[u8]) {
    OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
        .expect("Failed to open writing file.")
        .write(bytes)
        .expect("Failed to write bytes to path.");
}

fn from_year_to_months(value: f64) -> f64 {
    value / 12f64
}
