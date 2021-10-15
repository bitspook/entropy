use futures::stream::{self, StreamExt};
use futures::Stream;
use serde::de::DeserializeOwned;
use std::{
    fs::{self, DirEntry},
    io,
    path::Path,
};

pub trait HasTomlFMatter<T> {}

/// content_field is the name of the field in `T` in which content i.e
/// everything after toml frontmatter is put. Toml frontmatter is separated
/// from content with an empty line containing only "---"
pub fn from_toml_fmatter<T: DeserializeOwned>(
    input: &str,
    content_field: &str,
) -> Result<T, toml::de::Error> {
    let mut meta: Vec<&str> = vec![];
    let mut content: Vec<&str> = vec![];
    let lines = input.lines();
    let mut done_collecting_meta = false;

    for line in lines {
        if line.trim() == "---" {
            done_collecting_meta = true;
            continue;
        }

        if done_collecting_meta {
            content.push(line);
        } else {
            meta.push(line);
        }
    }

    let description = content.join("\n");
    let meta = meta.join("\n");
    let t = meta + &format!("\n{} = \"\"\"{}\"\"\"", content_field, description);

    toml::from_str(&t)
}

// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &Path, collector: &mut Vec<DirEntry>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, collector)?;
            } else {
                collector.push(entry);
            }
        }
    }

    Ok(())
}

/// Synchronously obtain directory listing recursively, and provide a stream of
/// `DirEntry`s
pub fn read_dir_recursively(dir: &Path) -> io::Result<impl Stream<Item = DirEntry>> {
    let mut paths: Vec<DirEntry> = vec![];

    visit_dirs(dir, &mut paths)?;

    Ok(stream::iter(paths))
}

// TODO: Return stream of futures and use buffering for parallelized reading
// files
pub async fn read_all_files_with_exts(
    exts: Vec<String>,
    base_dir: &Path,
) -> io::Result<impl Stream<Item = io::Result<String>>> {
    let listing = read_dir_recursively(base_dir)?
        .map(|de| de.path())
        .filter(move |fname| {
            futures::future::ready(
                fname
                    .extension()
                    .and_then(|e| e.to_str())
                    .map_or(false, |e| exts.contains(&e.to_string())),
            )
        })
        .map(|f| {
            debug!("Will read file [path={:?}]", f);
            f
        })
        .then(tokio::fs::read_to_string);

    Ok(listing)
}
