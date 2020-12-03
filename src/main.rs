//! Enerator
//!
//! A simple static site generator in Rust

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]

use clap::{load_yaml, App};
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag};
use std::fs;
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;

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
    let syntax_set = SyntaxSet::load_defaults_newlines();

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
                    let syntax = syntax_set
                        .find_syntax_by_token(&code_token)
                        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
                    println!("{}", syntax.name);
                    let mut html_generator = ClassedHTMLGenerator::new_with_class_style(
                        syntax,
                        &syntax_set,
                        ClassStyle::Spaced,
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

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();
    if let Some(matches) = matches.subcommand_matches("build") {
        if let Some(f) = matches.value_of("INPUT") {
            println!("{}", parse(f));
            println!("{}", f);
        }
    }

    let syntax_set = SyntaxSet::load_defaults_newlines();
    for syn in syntax_set.syntaxes() {
        println!("{}", syn.name);
        for ext in &syn.file_extensions {
            println!("  {}", ext);
        }
    }
}
