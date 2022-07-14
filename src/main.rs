use docgen::Docgen;
use generator::{DocgenGenerator, MarkdownGenerator};
use std::{collections::HashMap, fs, path::PathBuf};
use structopt::StructOpt;
use sv_parser::{Define, DefineText};

mod comment_parser;
mod docgen;
mod generator;
mod numbered_list;

#[derive(StructOpt)]
struct Opt {
    pub files: Vec<PathBuf>,

    /// Include path
    #[structopt(short = "i", long = "include", multiple = true, number_of_values = 1)]
    pub includes: Vec<PathBuf>,

    /// Define
    #[structopt(short = "d", long = "define", multiple = true, number_of_values = 1)]
    pub defines: Vec<String>,

    /// Quiet
    #[structopt(short = "q", long = "quiet")]
    pub quiet: bool,

    #[structopt(short = "o", long = "output")]
    pub output: Option<String>,
}

fn main() {
    let opt = Opt::from_args();

    let mut defines = HashMap::new();
    for define in &opt.defines {
        let mut define = define.splitn(2, '=');
        let ident = String::from(define.next().unwrap());
        let text = if let Some(x) = define.next() {
            let x = enquote::unescape(x, None).unwrap();
            Some(DefineText::new(x, None))
        } else {
            None
        };
        let define = Define::new(ident.clone(), vec![], text);
        defines.insert(ident, Some(define));
    }

    let mut result = vec![];
    for file in opt.files {
        let docgen = Docgen::from_file(file.to_str().unwrap(), &defines, &opt.includes).unwrap();
        result.push(docgen.parse_tree());
    }

    let md_str = MarkdownGenerator::generate(result);
    if let Some(output) = opt.output {
        fs::write(output, md_str).unwrap();
    } else {
        println!("{}", md_str);
    }
}
