use std::{fs, error::Error, fs::OpenOptions, io::Write};
use scraper::{Html, Selector};
use markdown::{to_html_with_options, Options as MarkOptions, ParseOptions, CompileOptions, Constructs};
use serde::Serialize;
#[derive(PartialEq)]
enum HeaderKind{
    Block,
    Child
}
#[derive(Serialize)]
struct HeaderBlock{
    title: String,
    text: String,
    children: Vec<String>
}

struct Header{
    kind: HeaderKind,
    text: String
}

fn parse_text(text:String)->String{ //remove emojis, whitespace, and lowercase the string
    let new_text = text.replace(|c: char| !c.is_ascii(), "");
    let new_text = new_text.to_lowercase().trim().to_string();
    new_text
}

fn parse_line(input_line: &str)->Option<Header>{
    let fragment = Html::parse_fragment(input_line);
    let h2_selector = Selector::parse("h2").unwrap();
    let h3_selector = Selector::parse("h3").unwrap();
    if let Some(h1) = fragment.select(&h2_selector).next(){
        let text = h1.text().collect::<String>();
        Some(Header{kind:HeaderKind::Block, text:parse_text(text)})
    }
    else if let Some(h2) = fragment.select(&h3_selector).next(){
        let text = h2.text().collect::<String>();
        Some(Header{kind:HeaderKind::Child, text:parse_text(text)})
    }else{
        None
    }
}

fn parse_document(document: &String)-> Vec<HeaderBlock>{
    let mut blocks = Vec::new();
    let mut curr_block = HeaderBlock{title:"introduction".to_string(), text: String::new(), children: vec![]};
    let mut lines = document.lines().peekable();
    while let Some(line) = lines.next(){
        if let Some(header) = parse_line(line){
            if header.kind == HeaderKind::Block{
                blocks.push(curr_block);
                curr_block = HeaderBlock{title: header.text, text: String::from(line), children: vec![]};
                break;
            }
        }
        curr_block.text.push_str(line);
    }
    while let Some(line) = lines.next(){
        if let Some(header) = parse_line(line){
            match header.kind{
                HeaderKind::Block => {
                    blocks.push(curr_block);
                    curr_block = HeaderBlock{title: header.text, text: String::from(line), children: vec![]};
                },
                HeaderKind::Child => {
                    let mut new_string = line.to_string();
                    new_string.insert_str(3, &format!(" id={}", curr_block.children.len()));
                    curr_block.text.push_str(&new_string);
                    while let Some(line) = lines.peek(){
                        if let Some(_) = parse_line(line){
                            break;
                        }
                        curr_block.text.push_str(line);
                        lines.next();
                    }
                    curr_block.children.push(header.text);
                },
            }
            continue;
        }
        curr_block.text.push_str(line);
    }
    blocks.push(curr_block);
    blocks

}


pub fn render_markdown(contents: &String)->Result<(String, String), Box<dyn Error>>{
    let html = to_html_with_options(&contents, &MarkOptions { parse: ParseOptions{
        constructs: Constructs::gfm(),..ParseOptions::default()
    }, compile: CompileOptions::default() }).unwrap();

    let blocks = parse_document(&html);
    
    Ok((html, serde_json::to_string(&blocks)?))
}

pub fn write_markdown()->Result<(), Box<dyn Error>>{
    let contents = fs::read_to_string(std::path::Path::new("./Tetris-Community/tetriscommunity.md"))?;
    let (html, blocks) = render_markdown(&contents)?;

    let mut mark_file = OpenOptions::new()
    .write(true)
    .truncate(true)
    .create(true)
    .open("./public/render/tetriscommunity.md")?;

    mark_file.write(contents.as_bytes())?;

    let mut html_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("./public/render/tetriscommunity.html")?;

    html_file.write(html.as_bytes())?;

    let mut json_file = 
    OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("./public/render/tetriscommunity.json")?;

    json_file.write(blocks.as_bytes())?;
    Ok(())
}