use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::{generate, Generator, Shell};
use serde::Serialize;
use std::{io, fmt, path::PathBuf, process};

use crate::clear_terminal_screen;

#[derive(Debug, Clone, Copy, ValueEnum, Serialize)]
pub enum Algorithm {
    Ahash,
    Blake3,
    Fxhash,
    SHA256,
    SHA512,
}

/// Display an enum in lowercase
impl fmt::Display for Algorithm {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let field: String = format!("{self:?}");
        write!(formatter, "{}", field.to_lowercase())
    }
}

impl Default for Algorithm {
    fn default() -> Self { Self::Blake3 }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ResultFormat {
    Json,
    Yaml,
    Personal,
}

// https://stackoverflow.com/questions/74068168/clap-rs-not-printing-colors-during-help
pub fn get_styles() -> clap::builder::Styles {

    let cyan   = anstyle::Color::Ansi(anstyle::AnsiColor::Cyan);
    let green  = anstyle::Color::Ansi(anstyle::AnsiColor::Green);
    let yellow = anstyle::Color::Ansi(anstyle::AnsiColor::Yellow);

    clap::builder::Styles::styled()
        .placeholder(
            anstyle::Style::new()
                .fg_color(Some(yellow))
        )
        .usage(
            anstyle::Style::new()
                .fg_color(Some(cyan))
                .bold()
        )
        .header(
            anstyle::Style::new()
                .fg_color(Some(cyan))
                .bold()
                .underline()
        )
        .literal(
            anstyle::Style::new()
                .fg_color(Some(green))
        )
}

// https://docs.rs/clap/latest/clap/struct.Command.html#method.help_template
const APPLET_TEMPLATE: &str = "\
{before-help}
{about-with-newline}
{usage-heading} {usage}

{all-args}
{after-help}";

#[derive(Parser, Debug)]
#[command(
    // Read from `Cargo.toml`
    author, version, about,
    long_about = None,
    next_line_help = true,
    help_template = APPLET_TEMPLATE,
    styles=get_styles(),
)]
pub struct Arguments {
    /// Choose the hash algorithm.
    #[arg(short('a'), long("algorithm"), value_enum, default_value_t = Algorithm::default())]
    pub algorithm: Algorithm,

    /// Clear the terminal screen before listing the duplicate files.
    #[arg(short('c'), long("clear_terminal"), default_value_t = false)] // action = ArgAction::SetTrue
    pub clear_terminal: bool,

    /// Prints full path of duplicate files, otherwise relative path.
    #[arg(short('f'), long("full_path"), default_value_t = false)]
    pub full_path: bool,

    /**
    If provided, outputs the completion file for given shell.

    ### How to generate shell completions for Z-shell:

    #### Example 1 (as a regular user):
    Generate completion_derive.zsh file with:

    ```console

        find_duplicate_files --generate=zsh > completion_derive.zsh

    ```

    Append the contents of the completion_derive.zsh file to the end of completion zsh file.

    ZSH completions are commonly stored in any directory listed in your `$fpath` variable.

    On Linux, view `$fpath` variable with:

    ```console

        echo $fpath | perl -nE 'say for split /\s+/'

    ```

    And then, execute:

    ```console

        compinit && zsh

    ```

    #### Example 2 (as a regular user):
    Generate completions to rustup and find_duplicate_files.

    Visible to only the regular user.

    ```console

        mkdir -p ~/.oh-my-zsh/functions

        rustup completions zsh > ~/.oh-my-zsh/functions/_rustup

        find_duplicate_files --generate=zsh > ~/.oh-my-zsh/functions/_find_duplicate_files

        compinit && zsh

    ```

    #### Example 3 (as root):

    Generate completions to rustup and find_duplicate_files.

    Visible to all system users.

    ```console

        mkdir -p /usr/local/share/zsh/site-functions

        rustup completions zsh > /usr/local/share/zsh/site-functions/_rustup

        find_duplicate_files --generate=zsh > /usr/local/share/zsh/site-functions/_find_duplicate_files

        compinit && zsh

    ```

    See `rustup completions` for detailed help.

    <https://github.com/clap-rs/clap/blob/master/clap_complete/examples/completion-derive.rs>
    */
    #[arg(short('g'), long("generate"), value_enum)]
    pub generator: Option<Shell>,

    /// Set the maximum depth to search for duplicate files.
    #[arg(short('m'), long("max_depth"), required = false)]
    pub max_depth: Option<usize>,

    /// Omit hidden files (starts with '.'), otherwise search all files.
    #[arg(short('o'), long("omit_hidden"), default_value_t = false)]
    pub omit_hidden: bool,

    /// Set the path where to look for duplicate files,
    /// otherwise use the current directory.
    #[arg(short('p'), long("path"), required = false)]
    pub path: Option<PathBuf>,

    /// Print the result in the chosen format.
    #[arg(short('r'), long("result_format"), value_enum, default_value_t = ResultFormat::Personal)]
    pub result_format: ResultFormat,

    /// Sort result by file size, otherwise sort by number of duplicate files.
    #[arg(short('s'), long("sort"), default_value_t = false)]
    pub sort: bool,

    /// Show total execution time.
    #[arg(short('t'), long("time"), default_value_t = false)]
    pub time: bool,

    /// Show intermediate runtime messages.
    #[arg(short('v'), long("verbose"), default_value_t = false)]
    pub verbose: bool,
}

impl Arguments {
    /// Build Arguments struct
    pub fn build() -> Arguments {
        let args: Arguments = Arguments::parse();

        if let Some(generator) = args.generator {
            args.print_completions(generator);
        }

        if args.clear_terminal {
            clear_terminal_screen();
        }

        args
    }

    /// Print shell completions to standard output
    fn print_completions<G>(&self, gen: G)
    where
        G: Generator + std::fmt::Debug,
    {
        let mut cmd = Arguments::command();
        let cmd_name = cmd.get_name().to_string();
        let mut stdout = io::stdout();

        eprintln!("Generating completion file for {gen:?}...");
        generate(gen, &mut cmd, cmd_name, &mut stdout);
        process::exit(1);
    }
}
