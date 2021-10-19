use futures::stream::{self, StreamExt};
use futures::Stream;
use regex::Regex;
use std::{
    fs::{self, DirEntry},
    io,
    path::Path,
};
use toml::Value;

#[derive(Debug)]
pub struct FMatterSection {
    pub meta: Value,
    pub content: String,
}

/// content_field is the name of the field in `T` in which content i.e
/// everything after toml frontmatter is put. Toml frontmatter is separated
/// from content with an empty line containing only "---"
pub fn into_toml_fmatter_sections<'a>(
    input: &'a str,
) -> Result<Vec<FMatterSection>, toml::de::Error> {
    let mut sections = input.split("---\n");
    let mut result: Vec<FMatterSection> = vec![];

    while let Some(meta) = sections.next() {
        let content = sections.next().map_or("".to_string(), |s| s.to_string());
        let meta = toml::from_str(meta)?;

        result.push(FMatterSection { meta, content });
    }

    Ok(result)
}

/// Walk `dir` and collect files into `collector`
fn list_all_files(dir: &Path, collector: &mut Vec<DirEntry>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                list_all_files(&path, collector)?;
            } else {
                collector.push(entry);
            }
        }
    }

    Ok(())
}

/// Synchronously obtain directory listing recursively, and provide a stream of
/// `DirEntry`s
fn stream_all_files(dir: &Path) -> io::Result<impl Stream<Item = DirEntry>> {
    let mut paths: Vec<DirEntry> = vec![];

    list_all_files(dir, &mut paths)?;

    Ok(stream::iter(paths))
}

// TODO: Return stream of futures and use buffering for parallelized reading
// files
pub async fn read_all_files(
    base_dir: &Path,
    include_re: Option<Regex>,
    exclude_re: Option<Regex>,
) -> io::Result<impl Stream<Item = io::Result<String>>> {
    let content = stream_all_files(base_dir)?
        .map(|de| de.path())
        .filter(move |fname| {
            let mut should_read = false;

            if let Some(fname) = fname.to_str() {
                // FIXME Shouldn't need to clone here
                should_read = include_re
                    .clone()
                    .map_or(should_read, |re| re.is_match(fname));
                should_read = exclude_re
                    .clone()
                    .map_or(should_read, |re| re.is_match(fname));
            }

            futures::future::ready(should_read)
        })
        .then(|f| async {
            debug!("Will read file [path={:?}]", f);

            tokio::fs::read_to_string(f).await
        });

    Ok(content)
}
