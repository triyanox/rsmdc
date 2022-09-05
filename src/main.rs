use clap::Parser;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    filename: String,
    #[clap(short, long, value_parser)]
    save: Option<String>,
    #[clap(short, long, value_parser, default_value_t = 1)]
    count: u8,
}

fn get_markdown_tree(file_name: &str) -> String {
    let mut file = File::open(file_name).expect("file not found");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    return contents;
}

fn write_html_in_path(html: &str, path: &str) -> std::io::Result<()> {
    let mut file = File::create(format!("{}/converted.html", path.to_owned())).unwrap();
    match file.write_all(html.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

#[derive(Debug, Clone)]
struct HTMLElemnt {
    tag: String,
    attrs: Option<HashMap<String, String>>,
    childrens: String,
}
struct HTMLElemntList {
    elemnts: Vec<HTMLElemnt>,
    list_type: String,
}
impl HTMLElemntList {
    fn new(list_type: String) -> HTMLElemntList {
        HTMLElemntList {
            elemnts: Vec::new(),
            list_type,
        }
    }
    fn add(&mut self, elemnt: HTMLElemnt) {
        self.elemnts.push(elemnt);
    }
    fn clear(&mut self) {
        self.elemnts.clear();
    }
    fn to_html(&self) -> String {
        let mut html = String::new();
        html.push_str(&format!("<{}>", self.list_type));
        for elemnt in &self.elemnts {
            html.push_str(&elemnt.to_html());
        }
        html.push_str(&format!("</{}>", self.list_type));
        return html;
    }
}
#[derive(Clone)]
struct MarkdownElement {
    replacement: HTMLElemnt,
    regex: Regex,
}

impl HTMLElemnt {
    fn new(tag: String, attrs: Option<HashMap<String, String>>, childrens: String) -> HTMLElemnt {
        HTMLElemnt {
            tag,
            attrs,
            childrens,
        }
    }
    fn to_html(&self) -> String {
        let mut html = String::new();
        if self.tag == "emb" {
            html.push_str("<em><strong>");
            html.push_str(&self.childrens);
            html.push_str("</strong></em>");
        } else {
            html.push_str(&format!("<{}", self.tag));
            if let Some(attrs) = &self.attrs {
                for (key, value) in attrs.iter() {
                    html.push_str(&format!(" {}=\"{}\"", key, value));
                }
            }
            if self.tag == "img" {
                html.push_str(&format!(" alt=\"{}\"/>", self.childrens));
            } else if self.tag == "hr" {
                html.push_str(&format!(">"));
            } else {
                html.push_str(&format!(">{}</{}>", self.childrens, self.tag));
            }
        }
        return html;
    }
}
impl MarkdownElement {
    fn new(replacement: HTMLElemnt, regex: Regex) -> MarkdownElement {
        MarkdownElement { replacement, regex }
    }
}

struct Lexer {}
impl Lexer {
    fn new() -> Lexer {
        Lexer {}
    }
    fn parse(&self, markdown: &str) -> String {
        let mut ast = self.parse_line(markdown);
        ast = self.parse_ol(&ast);
        ast = self.parse_ul(&ast);
        ast = self.parse_styles(&ast);
        ast = self.parse_code(&ast);
        ast = self.parse_inline_code(&ast);
        ast = self.parse_image(&ast);
        ast = self.parse_link(&ast);
        ast
    }
    fn parse_line(&self, markdown: &str) -> String {
        let markdown_line_elements = vec![
            MarkdownElement::new(
                HTMLElemnt::new(String::from("h1"), Default::default(), String::from("")),
                Regex::new(r"^\s*# (.*)$").unwrap(),
            ),
            MarkdownElement::new(
                HTMLElemnt::new(String::from("h2"), Default::default(), String::from("")),
                Regex::new(r"^\s*## (.*)$").unwrap(),
            ),
            MarkdownElement::new(
                HTMLElemnt::new(String::from("h3"), Default::default(), String::from("")),
                Regex::new(r"^\s*### (.*)$").unwrap(),
            ),
            MarkdownElement::new(
                HTMLElemnt::new(String::from("h4"), Default::default(), String::from("")),
                Regex::new(r"^\s*#### (.*)$").unwrap(),
            ),
            MarkdownElement::new(
                HTMLElemnt::new(String::from("h5"), Default::default(), String::from("")),
                Regex::new(r"^\s*##### (.*)$").unwrap(),
            ),
            MarkdownElement::new(
                HTMLElemnt::new(String::from("h6"), Default::default(), String::from("")),
                Regex::new(r"^\s*###### (.*)$").unwrap(),
            ),
            MarkdownElement::new(
                HTMLElemnt::new(
                    String::from("blockquote"),
                    Default::default(),
                    String::from(""),
                ),
                Regex::new(r"^\s*> (.*)$").unwrap(),
            ),
            MarkdownElement::new(
                HTMLElemnt::new(String::from("hr"), Default::default(), String::from("")),
                Regex::new(r"^\s*---$").unwrap(),
            ),
        ];
        let mut result = String::new();
        let markdown_lines = markdown.lines();
        let mut is_match = false;
        for line in markdown_lines {
            for line_element in markdown_line_elements.iter() {
                if line_element.regex.is_match(line) {
                    let mut childrens = String::new();
                    if line_element.replacement.tag != "hr" {
                        childrens = line_element
                            .regex
                            .captures(line)
                            .unwrap()
                            .get(1)
                            .unwrap()
                            .as_str()
                            .to_string();
                    }

                    result.push_str(
                        HTMLElemnt::new(
                            line_element.replacement.tag.clone(),
                            line_element.replacement.attrs.clone(),
                            childrens.to_string(),
                        )
                        .to_html()
                        .as_str(),
                    );
                    result.push_str("\n");
                    is_match = true;
                }
            }
            if is_match {
                is_match = false
            } else {
                result.push_str(line);
                result.push_str("\n");
            }
        }
        result
    }
    fn parse_code(&self, markdown: &str) -> String {
        let pre_element = MarkdownElement::new(
            HTMLElemnt::new(String::from("pre"), Default::default(), String::from("")),
            Regex::new(r"^```\s*(.*)$").unwrap(),
        );
        let mut result = String::new();
        let mut is_start = false;
        let mut is_end = false;
        let mut childrens = String::new();
        for word in markdown.split_whitespace() {
            if word == "```" || word.starts_with("```") || word.ends_with("```") {
                if is_start {
                    is_end = true;
                } else {
                    is_start = true;
                }
            }
            if is_start {
                if word != "```" {
                    if word.starts_with("```") && word.ends_with("```") {
                        childrens.push_str(&word[3..word.len() - 3]);
                        is_end = true;
                    } else if word.starts_with("```") {
                        childrens.push_str(&word[3..]);
                        childrens.push(' ');
                    } else if word.ends_with("```") {
                        childrens.push_str(&word[0..word.len() - 3]);
                    } else {
                        childrens.push_str(word);
                        childrens.push(' ');
                    }
                }
            } else {
                result.push_str(word);
                result.push(' ');
            }
            if is_end {
                is_start = false;
                is_end = false;
                result.push_str(
                    HTMLElemnt::new(
                        pre_element.replacement.tag.clone(),
                        pre_element.replacement.attrs.clone(),
                        childrens.to_string(),
                    )
                    .to_html()
                    .as_str(),
                );
                childrens = String::new();
            }
        }
        result
    }
    fn parse_inline_code(&self, markdown: &str) -> String {
        let code_elemet = MarkdownElement::new(
            HTMLElemnt::new(String::from("code"), Default::default(), String::from("")),
            Regex::new(r"^`\s*(.*)$").unwrap(),
        );
        let mut result = String::new();
        let mut is_start = false;
        let mut is_end = false;
        let mut childrens = String::new();
        for word in markdown.split_whitespace() {
            if word == "`" || word.starts_with("`") || word.ends_with("`") {
                if is_start {
                    is_end = true;
                } else {
                    is_start = true;
                }
            }
            if is_start {
                if word != "`" {
                    if word.starts_with("`") && word.ends_with("`") {
                        childrens.push_str(&word[1..word.len() - 1]);
                        is_end = true;
                    } else if word.starts_with("`") {
                        childrens.push_str(&word[1..]);
                        childrens.push(' ');
                    } else if word.ends_with("`") {
                        childrens.push_str(&word[0..word.len() - 1]);
                    } else {
                        childrens.push_str(word);
                        childrens.push(' ');
                    }
                }
            } else {
                result.push_str(word);
                result.push(' ');
            }
            if is_end {
                is_start = false;
                is_end = false;
                result.push_str(
                    HTMLElemnt::new(
                        code_elemet.replacement.tag.clone(),
                        code_elemet.replacement.attrs.clone(),
                        childrens.to_string(),
                    )
                    .to_html()
                    .as_str(),
                );
                childrens = String::new();
            }
        }
        result
    }
    fn parse_image(&self, markdown: &str) -> String {
        let image_element = MarkdownElement::new(
            HTMLElemnt::new(String::from("img"), Default::default(), String::from("")),
            Regex::new(r"!\[(.*)\]\((.*)\)").unwrap(),
        );
        let mut result = String::new();

        let mut words = markdown.split_whitespace();
        while let Some(word) = words.next() {
            if image_element.regex.is_match(word) {
                let captures = image_element.regex.captures(word).unwrap();
                result.push_str(
                    HTMLElemnt::new(
                        image_element.replacement.tag.clone(),
                        {
                            let mut attrs = HashMap::new();
                            attrs.insert(
                                String::from("src"),
                                captures.get(2).unwrap().as_str().to_string(),
                            );
                            Some(attrs)
                        },
                        captures.get(1).unwrap().as_str().to_string(),
                    )
                    .to_html()
                    .as_str(),
                )
            } else {
                result.push_str(word);
                result.push(' ');
            }
        }
        result
    }
    fn parse_link(&self, markdown: &str) -> String {
        let link_element = MarkdownElement::new(
            HTMLElemnt::new(String::from("a"), Default::default(), String::from("")),
            Regex::new(r"\[(.*)\]\((.*)\)").unwrap(),
        );
        let mut result = String::new();

        let mut words = markdown.split_whitespace();
        while let Some(word) = words.next() {
            if link_element.regex.is_match(word) {
                let captures = link_element.regex.captures(word).unwrap();
                result.push_str(
                    HTMLElemnt::new(
                        link_element.replacement.tag.clone(),
                        {
                            let mut attrs = HashMap::new();
                            attrs.insert(
                                String::from("href"),
                                captures.get(2).unwrap().as_str().to_string(),
                            );
                            Some(attrs)
                        },
                        captures.get(1).unwrap().as_str().to_string(),
                    )
                    .to_html()
                    .as_str(),
                )
            } else {
                result.push_str(word);
                result.push(' ');
            }
        }
        result
    }
    fn parse_ol(&self, markdown: &str) -> String {
        let mut ol_element = HTMLElemntList::new("ol".to_string());
        let li_element = MarkdownElement::new(
            HTMLElemnt::new(String::from("li"), Default::default(), String::from("")),
            Regex::new(r"^\s*\d+\.\s*(.*)$").unwrap(),
        );
        let mut result = String::new();
        let mut is_start = false;
        let mut is_end = false;
        let lines = markdown.lines();
        let last_line = lines.clone().last().unwrap();
        for line in lines {
            if li_element.regex.is_match(line) {
                is_start = true;
            }
            if !li_element.regex.is_match(line) && is_start {
                is_end = true;
            }
            if is_start {
                if li_element.regex.is_match(line) {
                    let captures = li_element.regex.captures(line).unwrap();
                    ol_element.add(HTMLElemnt::new(
                        li_element.replacement.tag.clone(),
                        li_element.replacement.attrs.clone(),
                        captures.get(1).unwrap().as_str().to_string(),
                    ))
                }
            } else {
                result.push_str(line);
                result.push_str("\n");
            }
            if is_start && line == last_line {
                is_end = true;
            }
            if is_end {
                is_start = false;
                is_end = false;
                result.push_str(ol_element.to_html().as_str());
                ol_element.clear();
            }
        }

        result
    }
    fn parse_ul(&self, markdown: &str) -> String {
        let mut ul_element = HTMLElemntList::new("ul".to_string());
        let li_element = MarkdownElement::new(
            HTMLElemnt::new(String::from("li"), Default::default(), String::from("")),
            Regex::new(r"^\s*-\s(.*)$").unwrap(),
        );
        let mut result = String::new();
        let mut is_start = false;
        let mut is_end = false;
        let lines = markdown.lines();
        let last_line = lines.clone().last().unwrap();
        for line in lines {
            if li_element.regex.is_match(line) {
                is_start = true;
            }
            if !li_element.regex.is_match(line) && is_start {
                is_end = true;
            }
            if is_start {
                if li_element.regex.is_match(line) {
                    let captures = li_element.regex.captures(line).unwrap();
                    ul_element.add(HTMLElemnt::new(
                        li_element.replacement.tag.clone(),
                        li_element.replacement.attrs.clone(),
                        captures.get(1).unwrap().as_str().to_string(),
                    ))
                }
            } else {
                result.push_str(line);
                result.push_str("\n");
            }
            if is_start && line == last_line {
                is_end = true;
            }
            if is_end {
                is_start = false;
                is_end = false;
                result.push_str(ul_element.to_html().as_str());
                ul_element.clear();
            }
        }

        result
    }
    fn parse_styles(&self, markdown: &str) -> String {
        let bold_element = MarkdownElement::new(
            HTMLElemnt::new(String::from("strong"), Default::default(), String::from("")),
            Regex::new(r"\*\*(.*)\*\*").unwrap(),
        );
        let italic_element = MarkdownElement::new(
            HTMLElemnt::new(String::from("em"), Default::default(), String::from("")),
            Regex::new(r"_(.*)_").unwrap(),
        );
        let bold_italic_element = MarkdownElement::new(
            HTMLElemnt::new(String::from("emb"), Default::default(), String::from("")),
            Regex::new(r"\*\*\*(.*)\*\*\*").unwrap(),
        );
        let marked_element = MarkdownElement::new(
            HTMLElemnt::new(String::from("mark"), Default::default(), String::from("")),
            Regex::new(r"===(.*)===").unwrap(),
        );
        let strikethrough_element = MarkdownElement::new(
            HTMLElemnt::new(String::from("del"), Default::default(), String::from("")),
            Regex::new(r"~~(.*)~~").unwrap(),
        );
        let mut result = String::new();
        let words = markdown.split_whitespace();
        for word in words {
            if bold_italic_element.regex.is_match(word) {
                let captures = bold_italic_element.regex.captures(word).unwrap();
                result.push_str(
                    HTMLElemnt::new(
                        bold_italic_element.replacement.tag.clone(),
                        bold_italic_element.replacement.attrs.clone(),
                        captures.get(1).unwrap().as_str().to_string(),
                    )
                    .to_html()
                    .as_str(),
                );
                result.push_str(" ");
            } else if italic_element.regex.is_match(word) {
                let captures = italic_element.regex.captures(word).unwrap();
                result.push_str(
                    HTMLElemnt::new(
                        italic_element.replacement.tag.clone(),
                        italic_element.replacement.attrs.clone(),
                        captures.get(1).unwrap().as_str().to_string(),
                    )
                    .to_html()
                    .as_str(),
                );
                result.push_str(" ");
            } else if bold_element.regex.is_match(word) {
                let captures = bold_element.regex.captures(word).unwrap();
                result.push_str(
                    HTMLElemnt::new(
                        bold_element.replacement.tag.clone(),
                        bold_element.replacement.attrs.clone(),
                        captures.get(1).unwrap().as_str().to_string(),
                    )
                    .to_html()
                    .as_str(),
                );
                result.push_str(" ");
            } else if marked_element.regex.is_match(word) {
                let captures = marked_element.regex.captures(word).unwrap();
                result.push_str(
                    HTMLElemnt::new(
                        marked_element.replacement.tag.clone(),
                        marked_element.replacement.attrs.clone(),
                        captures.get(1).unwrap().as_str().to_string(),
                    )
                    .to_html()
                    .as_str(),
                );
                result.push_str(" ");
            } else if strikethrough_element.regex.is_match(word) {
                let captures = strikethrough_element.regex.captures(word).unwrap();
                result.push_str(
                    HTMLElemnt::new(
                        strikethrough_element.replacement.tag.clone(),
                        strikethrough_element.replacement.attrs.clone(),
                        captures.get(1).unwrap().as_str().to_string(),
                    )
                    .to_html()
                    .as_str(),
                );
                result.push_str(" ");
            } else {
                result.push_str(word);
                result.push(' ');
            }
        }
        result
    }
}

struct Builder {
    html: String,
}
impl Builder {
    fn new(html: String) -> Builder {
        Builder { html }
    }
    fn build(&self) -> String {
        let build = format!(
            "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>Markdown</title></head><body>"
        )
        .to_string()
            + self.html.as_str()
            + "</body></html>";

        build.to_string()
    }
}
fn main() {
    let args = Args::parse();
    for _ in 0..args.count {
        if args.save != None {
            let tree = get_markdown_tree(&args.filename);
            let lexer = Lexer::new();
            let html = lexer.parse(&tree);
            let builder = Builder::new(html);
            match write_html_in_path(&builder.build(), &args.save.clone().unwrap()) {
                Ok(_) => println!("File saved in {}", args.save.as_ref().unwrap()),
                Err(e) => println!("Error: {}", e),
            }
        } else {
            let tree = get_markdown_tree(&args.filename);
            let lexer = Lexer::new();
            let html = lexer.parse(&tree);
            let builder = Builder::new(html);
            let build = builder.build();
            println!("{}", build);
        }
    }
}
