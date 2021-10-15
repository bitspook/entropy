use async_stream::stream;
use futures::stream::StreamExt;
use futures::Stream;
use serde::de::DeserializeOwned;
use std::{
    io,
    path::{Path, PathBuf},
};
use tokio::fs::read_dir;

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

// TODO: Make it work recursively
pub async fn walk_dir_recursively(dir: &Path) -> io::Result<impl Stream<Item = PathBuf>> {
    let mut listing = read_dir(dir).await?;

    let file_stream = stream! {
        while let Some(entry) = listing.next_entry().await.unwrap() {
            let file_type = entry.file_type().await.unwrap();

            if file_type.is_dir() {
                info!(
                    "Encountered a sub-directory. Gotta dig deeper into [dir={:?}]",
                    entry.path()
                );
                continue;
            }

            yield(entry.path());
        }
    };

    Ok(file_stream)
}

// TODO: Return stream of futures and use buffering for parallelized reading
// files
pub async fn read_all_files_with_exts(
    exts: Vec<String>,
    base_dir: &Path,
) -> io::Result<impl Stream<Item = io::Result<String>>> {
    let listing = walk_dir_recursively(base_dir)
        .await?
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
