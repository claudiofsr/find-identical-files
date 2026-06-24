#![allow(unused)]

use std::fmt::Display;

// https://gist.github.com/abritinthebay/d80eb99b2726c83feb0d97eab95206c4
// https://talyian.github.io/ansicolors/

// Ansi Colors:
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";

pub trait Colors {
    fn bold(&self) -> String;
    fn red(&self) -> String;
    fn green(&self) -> String;
    fn yellow(&self) -> String;
    fn blue(&self) -> String;
}

impl<T> Colors for T
where
    T: Display, // + Deref<Target = str>
{
    fn bold(&self) -> String {
        format!("{BOLD}{self}{RESET}")
    }

    fn red(&self) -> String {
        format!("{RED}{self}{RESET}")
    }

    fn green(&self) -> String {
        format!("{GREEN}{self}{RESET}")
    }

    fn yellow(&self) -> String {
        format!("{YELLOW}{self}{RESET}")
    }

    fn blue(&self) -> String {
        format!("{BLUE}{self}{RESET}")
    }
}
