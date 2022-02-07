use std::env;
use std::iter::Inspect;
use std::process;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

enum Opcode {
    OP_PUSH,
    OP_POP,
    OP_ADD,
    OP_MINUS,
    OP_DUMP
}

struct Instruction {
    opcode: Opcode,
    operands: Vec<i64>
}

impl Instruction {
    fn new(opcode: Opcode, operands: Vec<i64>) -> Self {
        Instruction { opcode, operands }
    }
}

fn usage() {
    println!("{} v{}", NAME.to_uppercase(), VERSION);
    println!("A Forth-like programming language written in Rust");
    println!("\nUSAGE:");
    println!("    {} <SUBCOMMAND> [OPTIONS]", env::current_exe().unwrap().file_name().unwrap().to_str().unwrap());
    println!("\nOPTIONS:");
    println!("    -h, --help                    Print this message");
    println!("\nSUBCOMMANDS:");
    println!("    interpret <FILE>              Interprets source file FILE");
    println!("    compile <FILE> [-r, --run]    Compiles source file FILE into native code");
    println!("        -r, --run                 Runs program after compiling");
}

fn main() {

    let mut comp : bool = false;
    let mut interp : bool = false;
    let mut run_prog : bool = false;
    let mut source_file : String = String::from("");
    let mut file_next : bool = false;

    for arg in env::args() {
        if arg == "-h" || arg == "--help" {
            usage();
            process::exit(0);
        }
        if arg == "compile" {
            comp = true;
            file_next = true;
            continue;
        }
        if arg == "interpret" {
            interp = true;
            file_next = true;
            continue;
        }
        if arg == "-r" || arg == "--run" {
            run_prog = true;
        }
        if file_next {
            source_file = arg;
        }
    }

    if source_file.is_empty() {
        usage();
        println!("\n[ERROR] Missing source file.");
        process::exit(1);
    }

    if interp && comp {
        usage();
        println!("\n[ERROR] `compile` and `interpret` subcommands are mutually exclusive.");
        process::exit(1);
    }

    println!("[INFO] source_file: {:?}", source_file);
    if interp {
        interpret(source_file.as_str());
    }
    if comp {
        compile(source_file.as_str());
    }
}

fn interpret(file : &str) {
    let program : Vec<Instruction> = vec![
        Instruction::new(Opcode::OP_PUSH, vec![34]),
        Instruction::new(Opcode::OP_PUSH, vec![35]),
        Instruction::new(Opcode::OP_ADD, vec![]),
        Instruction::new(Opcode::OP_DUMP, vec![])
    ];

    let mut stack : Vec<i64> = Vec::new();
    for ins in program {
        match ins.opcode {
            Opcode::OP_PUSH => {
                stack.push(ins.operands[0]);
            },
            Opcode::OP_POP => {
                let a = stack.pop();
            },
            Opcode::OP_ADD => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(b+a);
            },
            Opcode::OP_MINUS => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(b-a);
            },
            Opcode::OP_DUMP => {
                println!("{}", stack.pop().unwrap());
            }
        }
    }
}

fn compile(file: &str) {
    println!("Not implemented");
}