use std::collections::HashMap;
use pulldown_cmark::{html, Options, Parser};
use serde_json::Value;
use rocket_dyn_templates::tera::{Result, try_get_value};

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
