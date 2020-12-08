//! Enerator
//!
//! A simple static site generator in Rust

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]

use clap::{load_yaml, App};
use lazy_static::lazy_static;
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag};
use std::fs;
use std::path::PathBuf;
use syntect::dumps::from_binary;
use syntect::highlighting::ThemeSet;
use syntect::html::{css_for_theme_with_class_style, ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;

lazy_static! {
    pub static ref SYNTAX_SET: SyntaxSet = {
        let ss: SyntaxSet = from_binary(include_bytes!("../syntax/newlines.packdump"));
        ss
    };
    pub static ref THEME_SET: ThemeSet = from_binary(include_bytes!("../syntax/all.themedump"));
}

const CLASS_STYLE: ClassStyle = ClassStyle::SpacedPrefixed { prefix: "syn-" };

/// Reads markdown text from a file and converts to HTML.
fn parse(filename: &str) -> String {
    let markdown_input =
        fs::read_to_string(filename).expect("Something went wrong reading the file");
    // Set up options and parser. Strikethroughs are not part of the CommonMark standard
    // and we therefore must enable it explicitly.
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    let parser = Parser::new_ext(&markdown_input, options);
    let mut new_parser = Vec::new();
    let mut code_token = String::with_capacity(12);
    let mut to_highlight = String::new();
    //let syntax_set = SyntaxSet::load_defaults_newlines();

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(cb)) => {
                if let CodeBlockKind::Fenced(ref token) = cb {
                    code_token = token.to_owned().into_string();
                }
            }
            Event::End(Tag::CodeBlock(_)) => {
                if !code_token.is_empty() {
                    println!("{}", code_token);
                    let syntax = SYNTAX_SET
                        .find_syntax_by_token(&code_token)
                        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
                    println!("{}", syntax.name);
                    let mut html_generator = ClassedHTMLGenerator::new_with_class_style(
                        syntax,
                        &SYNTAX_SET,
                        CLASS_STYLE,
                    );
                    for line in to_highlight.lines() {
                        html_generator.parse_html_for_line(line);
                    }
                    let html =
                        format!("<pre><code>{}</code></pre>", html_generator.finalize()).into();
                    new_parser.push(Event::Html(html));
                    code_token.clear();
                }
            }
            Event::Text(t) => {
                if code_token.is_empty() {
                    new_parser.push(Event::Text(t));
                } else {
                    to_highlight.push_str(&t);
                }
            }
            e => {
                new_parser.push(e);
            }
        }
    }

    // Write to String buffer.
    let mut html_output = String::new();
    html::push_html(&mut html_output, new_parser.into_iter());

    // Check that the output is what we expected.
    // println!("{}", html_output);
    html_output
}

fn syntaxes() {
    for syn in SYNTAX_SET.syntaxes() {
        println!("{}", syn.name);
        for ext in &syn.file_extensions {
            println!("  {}", ext);
        }
    }
}

fn themes() {
    for theme in THEME_SET.themes.keys() {
        println!("{}", theme);
    }
}

fn css(theme: &str) -> String {
    let mut styles = String::from("");
    if let Some(t) = THEME_SET.themes.get(theme) {
        styles = css_for_theme_with_class_style(t, CLASS_STYLE)
    }
    styles
}

fn write_css(theme: &str, dir: &str) {
    let styles = css(theme);
    let mut path = PathBuf::from(dir);
    fs::create_dir_all(&path).expect("Cannot create directory.");
    path.push(theme);
    path.set_extension("css");

    if let Some(p) = path.to_str() {
        println!("{}", p);
    }
    //let filename = format!("{}/{}.css", dir, theme);
    fs::write(path, styles).expect("Unable to write file");
}

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();
    if let Some(matches) = matches.subcommand_matches("build") {
        if let Some(f) = matches.value_of("INPUT") {
            println!("{}", parse(f));
        }
    }
    if matches.subcommand_matches("syntaxes").is_some() {
        syntaxes();
    }
    if matches.subcommand_matches("themes").is_some() {
        themes();
    }
    if let Some(matches) = matches.subcommand_matches("css") {
        if let Some(t) = matches.value_of("theme") {
            if let Some(d) = matches.value_of("directory") {
                write_css(t, d);
            } else {
                println!("{}", css(t));
            }
        } else if let Some(d) = matches.value_of("directory") {
            for theme in THEME_SET.themes.keys() {
                write_css(theme, d);
            }
        }
    }
}
