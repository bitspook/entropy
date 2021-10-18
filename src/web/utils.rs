use pulldown_cmark::{html, Options, Parser};

pub fn render_md(input: &str) -> String {
    let options = Options::all();

    let parser = Parser::new_ext(input, options);
    let mut output = String::new();

    html::push_html(&mut output, parser);

    output
}
