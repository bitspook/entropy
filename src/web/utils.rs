use futures::executor::block_on;
use pulldown_cmark::{html, Options, Parser};
use rocket_dyn_templates::tera::{try_get_value, Result};
use serde_json::Value;
use std::collections::HashMap;

use crate::storage::get_signed_url;

pub fn render_md(input: &str) -> String {
    let options = Options::all();

    let parser = Parser::new_ext(input, options);
    let mut output = String::new();

    html::push_html(&mut output, parser);

    output
}

pub fn render_md_tera_filter(input: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    let s = try_get_value!("render_md", "value", String, input);
    let md = render_md(&s);

    Ok(Value::String(md))
}

pub fn storage_url_for(args: &HashMap<String, Value>) -> Result<Value> {
    let path = args.get("path").expect("storage_url_for path is required");
    let path = try_get_value!("path", "path", String, path);

    let bucket = args.get("bucket").expect("storage_url_for bucket is required");
    let bucket = try_get_value!("bucket", "bucket", String, bucket);

    let url = block_on(get_signed_url(&bucket, &path, 7 * 24 * 60 * 60)).unwrap();

    Ok(Value::String(url))
}
