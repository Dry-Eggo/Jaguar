pub(crate) mod backend;
pub(crate) mod frontend;
use std::process::Command;

use backend::codegen::Generator;
use backend::parser;
use clap::Parser as OtherParser;
use frontend::lexer;
use lexer::{TokenType, Tokenizer};
#[derive(OtherParser)]
#[command(
    name = "Jagc",
    version = "0.1",
    author = "Dry",
    about = "Jaguar Compiler"
)]
#[derive(Debug, Clone)]
pub struct Cli {
    #[arg(short, long)]
    pub release: bool,

    #[arg(value_name = "SOURCE")]
    pub source: String,

    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<String>,

    #[arg(long, help = "Keep the C artifacts")]
    pub keepc: bool,
}

fn initbuilddir() -> String {
    let cwd = std::env::current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let builddir = format!("{cwd}/build");
    // println!("Current Directory: {cwd}");
    // println!("Current Build Directory: {builddir}");
    if !std::path::Path::exists(std::path::Path::new(&builddir)) {
        let _ = std::fs::create_dir(builddir.clone());
    }
    builddir
}

fn main() {
    let cli = Cli::parse();

    let input = std::fs::read_to_string(cli.source.clone()).expect("Unable to open File");
    let mut tokenizer = Tokenizer::new(&input);
    let mut tokens = Vec::new();
    loop {
        let tok = tokenizer.next_token();
        if let TokenType::Comment(_) = tok.kind {
            continue;
        }
        tokens.push(tok.clone());
        if tok.kind == TokenType::EOF {
            break;
        }
    }
    let b = initbuilddir();
    let mut parser = parser::Parser::new(tokens, input.clone());
    match parser.parse_program() {
        program => {
            let mut cgen = Generator::new(
                program,
                &format!("{b}/{}", cli.output.clone().unwrap()),
                input,
                false,
                std::path::Path::new(cli.source.clone().as_str())
                    .to_str()
                    .unwrap()
                    .to_string(),
                std::fs::canonicalize(cli.source)
                    .ok()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap()
                    .to_string(),
                b,
            );
            cgen.init();
            cgen.generate(cgen.source.clone());
            // exit(1);
            cgen.rest();
            let mut gcc = Command::new("cc")
                .arg(cgen.outfilename)
                .arg("-o")
                .arg(cli.output.unwrap().to_string().clone())
                .arg("/home/dry/Documents/Eggo/jaguar/std/claw.o")
                .arg("/home/dry/Documents/Eggo/jaguar/std/stdjr.o")
                .arg("-no-pie")
                .arg("-w")
                .status();
            if !cli.keepc {
                std::fs::remove_dir_all(cgen.buildpath.to_str().unwrap()).unwrap();
            }
        }
    };
}
