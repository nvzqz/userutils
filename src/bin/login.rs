#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
extern crate liner;
extern crate termion;
extern crate redox_users;
extern crate userutils;

use std::fs::File;
use std::io::{self, Write};
use std::process::exit;
use std::env;
use std::str;

use extra::option::OptionalExt;
use arg_parser::ArgParser;
use termion::input::TermRead;
use redox_users::{AllUsers};
use userutils::spawn_shell;

const MAN_PAGE: &'static str = /* @MANSTART{login} */ r#"
NAME
    login - log into the computer

SYNOPSIS
    login

DESCRIPTION
    The login utility logs users (and pseudo-users) into the computer system.

OPTIONS

    -h
    --help
        Display this help and exit.

AUTHOR
    Written by Jeremy Soller, Jose Narvaez.
"#; /* @MANEND */

const ISSUE_FILE: &'static str = "/etc/issue";
const MOTD_FILE: &'static str = "/etc/motd";

pub fn main() {
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"]);
    parser.parse(env::args());

    // Shows the help
    if parser.found("help") {
        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    if let Ok(mut issue) = File::open(ISSUE_FILE) {
        io::copy(&mut issue, &mut stdout).try(&mut stderr);
        stdout.flush().try(&mut stderr);
    }

    loop {
        let user = liner::Context::new()
            .read_line("\x1B[1mredox login:\x1B[0m ", &mut |_| {})
            .try(&mut stderr);

        if !user.is_empty() {
            let stdin = io::stdin();
            let mut stdin = stdin.lock();
            let sys_users = AllUsers::new().unwrap_or_exit(1);

            match sys_users.get_by_name(user) {
                None => {
                    stdout.write(b"\nLogin incorrect\n").try(&mut stderr);
                    stdout.write(b"\n").try(&mut stderr);
                    stdout.flush().try(&mut stderr);
                    continue;
                },
                Some(user) => {
                    if user.is_passwd_blank() {
                        if let Ok(mut motd) = File::open(MOTD_FILE) {
                            io::copy(&mut motd, &mut stdout).try(&mut stderr);
                            stdout.flush().try(&mut stderr);
                        }

                        spawn_shell(user).unwrap_or_exit(1);
                        break;
                    }

                    stdout.write_all(b"\x1B[1mpassword:\x1B[0m ").try(&mut stderr);
                    stdout.flush().try(&mut stderr);
                    if let Some(password) = stdin.read_passwd(&mut stdout).try(&mut stderr) {
                        stdout.write(b"\n").try(&mut stderr);
                        stdout.flush().try(&mut stderr);

                        if user.verify_passwd(&password) {
                            if let Ok(mut motd) = File::open(MOTD_FILE) {
                                io::copy(&mut motd, &mut stdout).try(&mut stderr);
                                stdout.flush().try(&mut stderr);
                            }

                            spawn_shell(user).unwrap_or_exit(1);
                            break;
                        }
                    }
                }
            }
        } else {
            stdout.write(b"\n").try(&mut stderr);
            stdout.flush().try(&mut stderr);;
        }
    }
}
