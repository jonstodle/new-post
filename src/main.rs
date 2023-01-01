use chrono::{DateTime, Local, NaiveTime};
use clap::Parser;
use std::env::current_dir;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Title of the post (also used to derive file name)
    title: String,

    /// Tags to add ot the front matter
    tags: Vec<String>,

    /// Command to run to open the newly created file
    #[arg(short, long)]
    editor: Option<String>,
}

fn main() -> Result<(), Error> {
    let args = Arguments::parse();
    println!("{:?}", args);
    let today = Local::now()
        .date()
        .and_time(NaiveTime::default())
        .expect("NaiveTime should provide valid time");

    let content_dir = locate_content_directory()?;

    let new_file_path = content_dir.join(format!("{}.md", create_safe_file_name(&args.title)));

    write_file_contents(&args.title, today, args.tags, new_file_path.as_path())?;

    let editor = get_editor_command_string(args.editor)?;

    run_editor(editor, new_file_path.as_path())?;


    Ok(())
}

fn locate_content_directory() -> Result<PathBuf, Error> {
    let current_dir = current_dir()
        .map_err(|e| Error::from_error("Failed to get current working directory", &e))?;

    let content_directory_name = OsStr::new("content");
    if current_dir.file_name() == Some(content_directory_name) {
        return Ok(current_dir);
    }

    current_dir
        .read_dir()
        .map_err(|e| Error::from_error("Failed to get children of current working directory", &e))?
        .filter_map(|c| {
            if let Ok(de) = c {
                if de.file_type().ok()?.is_dir() {
                    Some(de)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .find(|dir| dir.file_name() == content_directory_name)
        .ok_or(Error::from_string(
            format!(
                "Failed to find a directory named '{}'",
                content_directory_name.to_string_lossy()
            )
            .as_str(),
        ))
        .map(|de| de.path())
}

fn write_file_contents(
    title: &str,
    date: DateTime<Local>,
    tags: Vec<String>,
    file_path: &Path,
) -> Result<(), Error> {
    let file_contents = format!(
        r#"+++
title = "{title}"
date = {date}
[taxonomies]
tags = [{tags}]
+++
"#,
        title = title,
        date = date.to_rfc3339(),
        tags = tags
            .iter()
            .map(|s| format!(r#""{}""#, s))
            .collect::<Vec<_>>()
            .join(", "),
    );

    fs::write(new_file_path.as_path(), file_contents)
        .map(|_| ())
        .map_err(|e| Error::from_error("Failed to create file", &e))
}

fn create_safe_file_name(title: &str) -> String {
    title.replace(&['\'', '"', '(', ')'], "")
}

fn get_editor_command_string(editor_path: Option<String>) -> Result<String, Error> {
    if let Some(cmd) = editor_path {
        Ok(cmd)
    } else {
        env::var("VISUAL")
            .or_else(|_| env::var("EDITOR"))
            .map_err(|_| Error::from_string("Unable to find a valid path to an editor"))
    }
}

fn run_editor(editor: String, file_path: &Path) -> Result<(), Error> {
    let mut editor_args = editor.split(' ').collect::<Vec<_>>();
    editor_args.push(
        file_path
            .as_os_str()
            .to_str()
            .expect("path with no trixie characters"),
    );

    let mut command = Command::new(editor_args[0]);
    command.args(editor_args.iter().skip(1));

    command
        .spawn()
        .map(|_| ())
        .map_err(|e| Error::from_error("Failed to start editor process", &e))?
        .wait()
}
}

#[derive(Debug)]
struct Error {
    message: String,
}

impl Error {
    fn from_error(message: &str, error: &dyn Display) -> Self {
        Error {
            message: format!("{}: {}", message, error),
        }
    }

    fn from_string(message: &str) -> Self {
        Error {
            message: message.to_string(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
