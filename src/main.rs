use docgen::Docgen;
use generator::{DocgenGenerator, MarkdownGenerator};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
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

    #[structopt(long = "wavedrom")]
    pub wavedrom: Option<String>,

    #[structopt(long = "graphviz")]
    pub graphviz: Option<String>,
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

    let cwd = if opt.output.is_none() {
        "./".to_string()
    } else {
        let x = opt.output.clone().unwrap();
        let x = PathBuf::from(x);
        let x = x.parent();
        String::from(x.unwrap().to_string_lossy())
        // x.unwrap().to_str().unwrap()
    };

    let md_gen = MarkdownGenerator::new(cwd.to_string(), opt.wavedrom, opt.graphviz);
    let md_str = md_gen.generate(result);
    if let Some(output) = &opt.output {
        fs::write(output, md_str).unwrap();
    } else {
        println!("{}", md_str);
    }
}
