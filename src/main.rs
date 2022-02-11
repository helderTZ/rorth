use std::{env};
use std::fs::File;
use std::io::Write;
use std::process;
use std::process::{Command, Stdio};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Debug, Clone)]
enum Opcode {
    OP_PUSH,
    OP_ADD,
    OP_SUB,
    OP_MUL,
    OP_DIV,
    OP_EQ,
    OP_NE,
    OP_GT,
    OP_LT,
    OP_GE,
    OP_LE,
    OP_DUP,
    OP_DUMP,
    OP_IF,
    OP_ELSE,
    OP_END,
    OP_WHILE,
    OP_DO,
}

#[derive(Debug, Clone)]
struct Instruction {
    opcode: Opcode,
    operands: Vec<i64>,
    ip: usize
}

impl Instruction {
    fn new(opcode: Opcode, operands: Vec<i64>, ip: usize) -> Self {
        Instruction { opcode, operands, ip}
    }
}

struct Token {
    tok: String,
    row: usize,
    col: usize
}

impl Token {
    fn new(tok: String, row: usize, col: usize) -> Self {
        Token { tok, row, col}
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
            continue;
        }
        if file_next {
            source_file = arg;
            continue;
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

    let tokens = lexer(source_file.as_str());
    let mut program = parser(&tokens);
    parser_second_pass(source_file.as_str(), &tokens, &mut program);
    if interp {
        interpret(&program);
    }
    if comp {
        compile(&program, run_prog);
    }
}

fn lexer(filename: &str) -> Vec<Token> {
    let source : String = std::fs::read_to_string(filename).unwrap();
    let mut tokens : Vec<Token> = Vec::new();
    for (i, line) in source.lines().enumerate() {
        for (j, tok) in line.split_whitespace().enumerate() {
            tokens.push(Token::new(tok.to_string(), i, j));
        }
    }
    tokens
}

fn parser(tokens : &Vec<Token>) -> Vec<Instruction> {
    let mut program : Vec<Instruction> = Vec::new();
    for (ip, tok) in tokens.iter().enumerate() {
        if tok.tok == "+"           { program.push(Instruction::new(Opcode::OP_ADD, vec![], ip)); }
        else if tok.tok == "-"      { program.push(Instruction::new(Opcode::OP_SUB, vec![], ip)); }
        else if tok.tok == "*"      { program.push(Instruction::new(Opcode::OP_MUL, vec![], ip)); }
        else if tok.tok == "/"      { program.push(Instruction::new(Opcode::OP_DIV, vec![], ip)); }
        else if tok.tok == "="      { program.push(Instruction::new(Opcode::OP_EQ, vec![], ip)); }
        else if tok.tok == "!="     { program.push(Instruction::new(Opcode::OP_NE, vec![], ip)); }
        else if tok.tok == ">"      { program.push(Instruction::new(Opcode::OP_GT, vec![], ip)); }
        else if tok.tok == ">="     { program.push(Instruction::new(Opcode::OP_GE, vec![], ip)); }
        else if tok.tok == "<"      { program.push(Instruction::new(Opcode::OP_LT, vec![], ip)); }
        else if tok.tok == "<="     { program.push(Instruction::new(Opcode::OP_LE, vec![], ip)); }
        else if tok.tok == "."      { program.push(Instruction::new(Opcode::OP_DUMP, vec![], ip)); }
        else if tok.tok == "dup"    { program.push(Instruction::new(Opcode::OP_DUP, vec![], ip)); }
        else if tok.tok == "if"     { program.push(Instruction::new(Opcode::OP_IF, vec![], ip)); }
        else if tok.tok == "end"    { program.push(Instruction::new(Opcode::OP_END, vec![], ip)); }
        else if tok.tok == "else"   { program.push(Instruction::new(Opcode::OP_ELSE, vec![], ip)); }
        else if tok.tok == "while"  { program.push(Instruction::new(Opcode::OP_WHILE, vec![], ip)); }
        else if tok.tok == "do"     { program.push(Instruction::new(Opcode::OP_DO, vec![], ip)); }
        else {
            let immediate = tok.tok.parse::<i64>().unwrap();
            program.push(Instruction::new(Opcode::OP_PUSH, vec![immediate], ip));
        }
    }
    program
}

fn dump_bytecode(program : &Vec<Instruction>) {
    println!("Bytecode:");
    for ins in program {
        println!("\t{:?}", ins);
    }
}

fn parser_second_pass(source_file : &str, tokens : &Vec<Token>, program : &mut Vec<Instruction>) {
    let mut stack : Vec<usize> = Vec::new();
    for ip in 0..tokens.len() {
        if program[ip].opcode == Opcode::OP_IF {
            stack.push(program[ip].ip);
        }
        if program[ip].opcode == Opcode::OP_END {
            if let Some(if_ip) = stack.pop() {
                let end_ip = program[ip].ip.clone();
                program[if_ip].operands.push(end_ip as i64);
            }
            else {
                println!("[ERROR] {}:{}:{}: Found `end` without matching `if`",
                    source_file, tokens[ip].row+1, tokens[ip].col+1);
                dump_bytecode(program);
                process::exit(1);
            }
        };
    }
}

fn interpret(program : &Vec<Instruction>) {
    let mut stack : Vec<i64> = Vec::new();
    let mut ip = 0;
    while ip < program.len() {
        let ins = &program[ip];
        match ins.opcode {
            Opcode::OP_PUSH => {
                stack.push(ins.operands[0]);
            },
            Opcode::OP_ADD => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a+b);
            },
            Opcode::OP_SUB => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(b-a);
            },
            Opcode::OP_MUL => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a*b);
            },
            Opcode::OP_DIV => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(b/a);
            },
            Opcode::OP_EQ => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(((a==b) as i32) as i64);
            },
            Opcode::OP_NE => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((a != b) as i64);
            },
            Opcode::OP_GT => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((a > b) as i64);
            },
            Opcode::OP_GE => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((a >= b) as i64);
            },
            Opcode::OP_LT => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((a < b) as i64);
            },
            Opcode::OP_LE => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((a <= b) as i64);
            },
            Opcode::OP_DUP => {
                let a = stack.pop().unwrap();
                stack.push(a);
                stack.push(a);
            },
            Opcode::OP_DUMP => {
                println!("{}", stack.pop().unwrap());
            }
            Opcode::OP_IF => {
                let a = stack.pop().unwrap();
                if a == 0 {
                    ip = ins.operands[0] as usize;
                }
            },
            Opcode::OP_ELSE => {
                unimplemented!();
            },
            Opcode::OP_END => { },
            Opcode::OP_WHILE => {
                unimplemented!();
            }
            Opcode::OP_DO => {
                unimplemented!();
            }
        }
        ip += 1;
    }
}

fn compile(program : &Vec<Instruction>, run_prog : bool) {
    codegen(program);
    build();
    if run_prog {
        execute();
    }
}

fn codegen(program: &Vec<Instruction>) {
    let mut asm_file = File::create("out.asm")
        .expect("Could not open file");
    writeln!(&mut asm_file, "%define SYS_EXIT 60").unwrap();
    writeln!(&mut asm_file, "%define SYS_WRITE 1").unwrap();
    writeln!(&mut asm_file, "section .text").unwrap();
    writeln!(&mut asm_file, "dump:").unwrap();
    writeln!(&mut asm_file, "    sub     rsp, 40").unwrap();
    writeln!(&mut asm_file, "    mov     rsi, rdi").unwrap();
    writeln!(&mut asm_file, "    mov  r10, -3689348814741910323").unwrap();
    writeln!(&mut asm_file, "    mov     BYTE [rsp+20], 10").unwrap();
    writeln!(&mut asm_file, "    lea     rcx, [rsp+19]").unwrap();
    writeln!(&mut asm_file, "    lea     r8, [rsp+21]").unwrap();
    writeln!(&mut asm_file, ".L2:").unwrap();
    writeln!(&mut asm_file, "    mov     rax, rsi").unwrap();
    writeln!(&mut asm_file, "    mov     r9, r8").unwrap();
    writeln!(&mut asm_file, "    mul     r10\n").unwrap();
    writeln!(&mut asm_file, "    mov     rax, rsi").unwrap();
    writeln!(&mut asm_file, "    sub     r9, rcx").unwrap();
    writeln!(&mut asm_file, "    shr     rdx, 3").unwrap();
    writeln!(&mut asm_file, "    lea     rdi, [rdx+rdx*4]").unwrap();
    writeln!(&mut asm_file, "    add     rdi, rdi").unwrap();
    writeln!(&mut asm_file, "    sub     rax, rdi").unwrap();
    writeln!(&mut asm_file, "    add     eax, 48").unwrap();
    writeln!(&mut asm_file, "    mov     BYTE [rcx], al").unwrap();
    writeln!(&mut asm_file, "    mov     rax, rsi").unwrap();
    writeln!(&mut asm_file, "    mov     rsi, rdx").unwrap();
    writeln!(&mut asm_file, "    mov     rdx, rcx").unwrap();
    writeln!(&mut asm_file, "    sub     rcx, 1").unwrap();
    writeln!(&mut asm_file, "    cmp     rax, 9").unwrap();
    writeln!(&mut asm_file, "    ja      .L2").unwrap();
    writeln!(&mut asm_file, "    sub     rdx, r8").unwrap();
    writeln!(&mut asm_file, "    mov     edi, 1").unwrap();
    writeln!(&mut asm_file, "    lea     rsi, [rsp+21+rdx]").unwrap();
    writeln!(&mut asm_file, "    mov     rdx, r9").unwrap();
    writeln!(&mut asm_file, "    mov     rax, SYS_WRITE").unwrap();
    writeln!(&mut asm_file, "    syscall").unwrap();
    writeln!(&mut asm_file, "    add     rsp, 40").unwrap();
    writeln!(&mut asm_file, "    ret").unwrap();
    writeln!(&mut asm_file, "global _start").unwrap();
    writeln!(&mut asm_file, "_start:").unwrap();
    for ins in program {
        match ins.opcode {
            Opcode::OP_PUSH => {
                writeln!(&mut asm_file, "    push {}", ins.operands[0]).unwrap();
            },
            Opcode::OP_ADD => {
                writeln!(&mut asm_file, "    pop rax").unwrap();
                writeln!(&mut asm_file, "    pop rbx").unwrap();
                writeln!(&mut asm_file, "    add rax, rbx").unwrap();
                writeln!(&mut asm_file, "    push rax").unwrap();
            },
            Opcode::OP_SUB => {
                writeln!(&mut asm_file, "    pop rax").unwrap();
                writeln!(&mut asm_file, "    pop rbx").unwrap();
                writeln!(&mut asm_file, "    sub rbx, rax").unwrap();
                writeln!(&mut asm_file, "    push rbx").unwrap();
            },
            Opcode::OP_MUL => {
                writeln!(&mut asm_file, "    pop rax").unwrap();
                writeln!(&mut asm_file, "    pop rbx").unwrap();
                writeln!(&mut asm_file, "    mul rbx").unwrap();
                writeln!(&mut asm_file, "    push rax").unwrap();
            },
            Opcode::OP_DIV => {
                //FIXME: not working
                writeln!(&mut asm_file, "    xor rdx, rdx").unwrap();
                writeln!(&mut asm_file, "    pop rbx").unwrap();
                writeln!(&mut asm_file, "    pop rax").unwrap();
                writeln!(&mut asm_file, "    div rbx").unwrap();
                writeln!(&mut asm_file, "    push rax").unwrap();
                writeln!(&mut asm_file, "    push rdx").unwrap();
            },
            Opcode::OP_EQ => {
                unimplemented!();
            },
            Opcode::OP_NE => {
                unimplemented!();
            },
            Opcode::OP_GT => {
                unimplemented!();
            },
            Opcode::OP_GE => {
                unimplemented!();
            },
            Opcode::OP_LT => {
                unimplemented!();
            },
            Opcode::OP_LE => {
                unimplemented!();
            },
            Opcode::OP_DUP => {
                writeln!(&mut asm_file, "    pop rax").unwrap();
                writeln!(&mut asm_file, "    push rax").unwrap();
                writeln!(&mut asm_file, "    push rax").unwrap();
            }
            Opcode::OP_DUMP => {
                writeln!(&mut asm_file, "    pop rdi").unwrap();
                writeln!(&mut asm_file, "    call dump").unwrap();
            },
            Opcode::OP_IF => {
                unimplemented!();
            },
            Opcode::OP_ELSE => {
                unimplemented!();
            },
            Opcode::OP_END => {
                unimplemented!();
            },
            Opcode::OP_WHILE => {
                unimplemented!();
            },
            Opcode::OP_DO => {
                unimplemented!();
            }
        }
    }
    writeln!(&mut asm_file, "    mov rax, SYS_EXIT").unwrap();
    writeln!(&mut asm_file, "    mov rdi, 0").unwrap();
    writeln!(&mut asm_file, "    syscall").unwrap();
    writeln!(&mut asm_file, "    ret").unwrap();
}

fn build() {
    let compiler_output = Command::new("nasm")
        .args(["-felf64", "out.asm"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    println!("[INFO] nasm compiler output: {}", String::from_utf8(compiler_output.stdout).unwrap());
    println!("[INFO] nasm compiler stderr: {}", String::from_utf8(compiler_output.stderr).unwrap());

    let linker_output = Command::new("ld")
        .args(["-o", "out", "out.o"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    println!("[INFO] ld linker output: {}", String::from_utf8(linker_output.stdout).unwrap());
    println!("[INFO] ld linker stderr: {}", String::from_utf8(linker_output.stderr).unwrap());
}

fn execute() {
    let program_output = Command::new("out")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    println!("[INFO] out output: {}", String::from_utf8(program_output.stdout).unwrap());
    println!("[INFO] out stderr: {}", String::from_utf8(program_output.stderr).unwrap());
}