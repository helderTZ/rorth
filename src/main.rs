use std::{env};
use std::fs::File;
use std::io::Write;
use std::process;
use std::process::{Command, Stdio};
use std::io;

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
    OP_NOT,
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

#[derive(Debug)]
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
    println!("    -h, --help                        Print this message");
    println!("    -b, --bytecode                    Dump bytecode to file");
    println!("\nSUBCOMMANDS:");
    println!("    interpret <FILE>                  Interprets source file FILE");
    println!("    compile <FILE> [-r] [-o OUT_FILE] Compiles source file FILE into native code");
    println!("        -r, --run                     Runs program after compiling");
    println!("        -o, --output                  Name of the executable (default: out)");
}

fn main() {

    let mut comp : bool = false;
    let mut interp : bool = false;
    let mut run_prog : bool = false;
    let mut dump_bc : bool = false;
    let mut exec_file: String = String::from("out");
    let mut source_file : String = String::from("");
    let mut source_file_next : bool = false;
    let mut exec_file_next : bool = false;

    for arg in env::args() {
        if arg == "-h" || arg == "--help" {
            usage();
            process::exit(0);
        }
        if arg == "compile" {
            comp = true;
            source_file_next = true;
            continue;
        }
        if arg == "interpret" {
            interp = true;
            source_file_next = true;
            continue;
        }
        if arg == "-r" || arg == "--run" {
            run_prog = true;
            continue;
        }
        if arg == "-b" || arg == "--bytecode" {
            dump_bc = true;
            continue;
        }
        if arg == "-o" || arg == "--output" {
            exec_file_next = true;
            continue;
        }
        if source_file_next {
            source_file = arg;
            continue;
        }
        if exec_file_next {
            exec_file = arg;
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
    let program = parser(&source_file, &tokens);

    if dump_bc {
        _dump_bytecode(&program);
        _dump_bytecode_to_file(&program, &source_file);
    }

    if interp {
        interpret(&program, &mut io::stdout());
    }
    if comp {
        compile(&program, &exec_file, run_prog);
    }
}

// debug function
fn _dump_tokens(tokens : &Vec<Token>) {
    println!("Tokens:");
    for (i, tok) in tokens.iter().enumerate() {
        println!("\t{} {:?}", i, tok);
    }
}

// debug function
fn _dump_bytecode(program : &Vec<Instruction>) {
    println!("Bytecode:\n[ip | opcode  | operands]");
    for (i, ins) in program.iter().enumerate() {
        println!("{:>3}   {:?}\t{:>?}", i, ins.opcode, ins.operands);
    }
}

// debug function
fn _dump_bytecode_to_file(program : &Vec<Instruction>, filename: &str) {
    let bytecode_filename = filename.to_string() + ".bytecode";
    let mut bytecode_file = File::create(bytecode_filename)
        .expect("Could not open file");
    for (i, ins) in program.iter().enumerate() {
        writeln!(&mut bytecode_file, "{:>3}   {:?} {:>?}", i, ins.opcode, ins.operands).unwrap();
    }
}

// debug function
fn _dump_crossref(stack: &Vec<usize>) {
    print!("Crossref:");
    for (i, val) in stack.iter().enumerate() {
        print!("({}, {}) ", i, val);
    }
    println!();
}

// debug function
fn _dump_stack(stack: &Vec<i64>) {
    print!("Stack:");
    for (i, val) in stack.iter().enumerate() {
        print!("({}, {}) ", i, val);
    }
    println!();
}

//FIXME: col is wrong, it should be the char index, not the word index
fn lexer(filename: &str) -> Vec<Token> {
    let source : String = std::fs::read_to_string(filename)
        .expect(&format!("Could not read file {}", filename));
    let mut tokens : Vec<Token> = Vec::new();
    for (i, line) in source.lines().enumerate() {
        let filtered_line = line.split("//").next().unwrap();
        for (j, tok) in filtered_line.split_whitespace().enumerate() {
            tokens.push(Token::new(tok.to_string(), i, j));
        }
    }
    tokens
}


/**
 * +---------------------+       +-------------------+
 * |        IF           |       |     WHILE    <-+  |
 * |    <condition> --+  |       |  <condition>   |  |
 * |        .     |   |  |       |      DO        |  |
 * |        .     |   |  |       |      .         |  |
 * |       ELSE <-+   |  |       |      .         |  |
 * |        .         |  |       |      .         |  |
 * |        .         |  |       |      .         |  |
 * |        .         |  |       |      .         |  |
 * |        END       |  |       |      END ------+  |
 * |           <------+  |       |                   |
 * |                     |       |                   |
 * +---------------------+       +-------------------+
 */
fn parser(source_file : &str, tokens : &Vec<Token>) -> Vec<Instruction> {
    let mut program : Vec<Instruction> = Vec::new();
    let mut crossref : Vec<usize> = Vec::new();
    for (ip, tok) in tokens.iter().enumerate() {
        if tok.tok == "+"           { program.push(Instruction::new(Opcode::OP_ADD, vec![], ip)); }
        else if tok.tok == "-"      { program.push(Instruction::new(Opcode::OP_SUB, vec![], ip)); }
        else if tok.tok == "*"      { program.push(Instruction::new(Opcode::OP_MUL, vec![], ip)); }
        else if tok.tok == "/"      { program.push(Instruction::new(Opcode::OP_DIV, vec![], ip)); }
        else if tok.tok == "!"      { program.push(Instruction::new(Opcode::OP_NOT, vec![], ip)); }
        else if tok.tok == "="      { program.push(Instruction::new(Opcode::OP_EQ, vec![], ip)); }
        else if tok.tok == "!="     { program.push(Instruction::new(Opcode::OP_NE, vec![], ip)); }
        else if tok.tok == ">"      { program.push(Instruction::new(Opcode::OP_GT, vec![], ip)); }
        else if tok.tok == ">="     { program.push(Instruction::new(Opcode::OP_GE, vec![], ip)); }
        else if tok.tok == "<"      { program.push(Instruction::new(Opcode::OP_LT, vec![], ip)); }
        else if tok.tok == "<="     { program.push(Instruction::new(Opcode::OP_LE, vec![], ip)); }
        else if tok.tok == "."      { program.push(Instruction::new(Opcode::OP_DUMP, vec![], ip)); }
        else if tok.tok == "dup"    { program.push(Instruction::new(Opcode::OP_DUP, vec![], ip)); }
        else if tok.tok == "if" {
            program.push(Instruction::new(Opcode::OP_IF, vec![], ip));
            crossref.push(ip);
        }
        else if tok.tok == "else" {
            program.push(Instruction::new(Opcode::OP_ELSE, vec![], ip));
            if let Some(if_ip) = crossref.pop() {
                if program[if_ip].opcode != Opcode::OP_IF {
                    eprintln!("[ERROR] {}:{}:{}: @ip {}: Found `else` without matching `if`",
                        source_file, tokens[ip].row+1, tokens[ip].col+1, ip);
                    _dump_bytecode(&program);
                    _dump_crossref(&crossref);
                    process::exit(1);
                }
                program[if_ip].operands.push(ip as i64);
                crossref.push(ip);
        } else {
                eprintln!("[ERROR] {}:{}:{}: @ip {}: Found `else` without matching `if`",
                    source_file, tokens[ip].row+1, tokens[ip].col+1, ip);
                _dump_bytecode(&program);
                _dump_crossref(&crossref);
                process::exit(1);
            }
        }
        else if tok.tok == "while" {
            program.push(Instruction::new(Opcode::OP_WHILE, vec![], ip));
            crossref.push(program[ip].ip);
        }
        else if tok.tok == "do" {
            if let Some(while_ip) = crossref.pop() {
                if program[while_ip].opcode != Opcode::OP_WHILE {
                    eprintln!("[ERROR] {}:{}:{}: @ip {}: Found `while` without matching `do`",
                        source_file, tokens[ip].row+1, tokens[ip].col+1, ip);
                    _dump_bytecode(&program);
                    _dump_crossref(&crossref);
                    process::exit(1);
                }
                program.push(Instruction::new(Opcode::OP_DO, vec![], ip));
                program[ip].operands.push(while_ip as i64);
                crossref.push(program[ip].ip);
            }
        }
        //TODO: support nested whiles
        else if tok.tok == "end" {
            // three situations
            // situation 1 : if -> end
            // situation 2 : if -> else -> end
            // situation 3 : while -> do -> end
            // --- situation 1 ---
            //      if points to end+1, end fallsthrough
            // --- situation 2 ---
            //      if points to else+1, else fallsthrough, end fallsthrough
            // --- situation 3 ---
            // do points to end+1, end points to while+1
            program.push(Instruction::new(Opcode::OP_END, vec![], ip));
            if let Some(prev_ip) = crossref.pop() {
                // situation 1 or 2
                if program[prev_ip].opcode == Opcode::OP_IF
                    || program[prev_ip].opcode == Opcode::OP_ELSE {
                    program[prev_ip].operands.push(ip as i64);
                }
                if program[prev_ip].opcode == Opcode::OP_WHILE {
                    eprintln!("[ERROR] {}:{}:{}: @ip {}: Found `while` without matching `do`",
                        source_file, tokens[ip].row+1, tokens[ip].col+1, ip);
                    _dump_bytecode(&program);
                    _dump_crossref(&crossref);
                    process::exit(1);
                }
                // situation 3, DO has WHILE's ip in its operands
                if program[prev_ip].opcode == Opcode::OP_DO {
                    if let Some(while_ip) = program[prev_ip].operands.pop() {
                        program[ip].operands.push(while_ip as i64);
                        program[prev_ip].operands.push(ip as i64);
                    } else {
                        eprintln!("[ERROR] {}:{}:{}: @ip {}:Found `do` without matching `while`",
                            source_file, tokens[ip].row+1, tokens[ip].col+1, ip);
                        _dump_bytecode(&program);
                        _dump_crossref(&crossref);
                        process::exit(1);
                    }
                }
            } else {
                eprintln!("[ERROR] {}:{}:{}: @ip {}: Found `end` without matching `if-else` or `while-do`",
                    source_file, tokens[ip].row+1, tokens[ip].col+1, ip);
                _dump_bytecode(&program);
                _dump_crossref(&crossref);
                process::exit(1);
            }
        }
        else {
            let immediate = tok.tok.parse::<i64>()
                .expect(&format!("[ERROR] {}:{}:{}: @ip {}: Expected integer, got {}",
                    source_file, tokens[ip].row+1, tokens[ip].col+1, ip, tok.tok));
            program.push(Instruction::new(Opcode::OP_PUSH, vec![immediate], ip));
        }
    }
    program
}

fn interpret<W: Write>(program : &Vec<Instruction>, stdout : &mut W) {
    // _dump_bytecode(program);
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
            Opcode::OP_NOT => {
                let a = stack.pop().unwrap();
                if a == 0 {
                    stack.push(1);
                } else if a == 1 {
                    stack.push(0);
                } else {
                    eprintln!("[ERROR] @ip {}: Expected a boolen in the stack, found {}", ip, a);
                    _dump_bytecode(&program);
                    _dump_stack(&stack);
                    process::exit(1);
                }
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
                stack.push((b > a) as i64);
            },
            Opcode::OP_GE => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((b >= a) as i64);
            },
            Opcode::OP_LT => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((b < a) as i64);
            },
            Opcode::OP_LE => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((b <= a) as i64);
            },
            Opcode::OP_DUP => {
                let a = stack.pop().unwrap();
                stack.push(a);
                stack.push(a);
            },
            Opcode::OP_DUMP => {
                if let Some(a) = stack.pop() {
                    writeln!(stdout, "{}", a).unwrap();
                } else {
                    eprintln!("[ERROR] @ip {}: Tried to pop but stack was empty", ip);
                    _dump_bytecode(&program);
                    _dump_stack(&stack);
                    process::exit(1);
                }
            }
            Opcode::OP_IF => {
                let a = stack.pop().unwrap();
                if a == 0 {
                    ip = ins.operands[0] as usize;
                }
            },
            Opcode::OP_ELSE => {
                ip = ins.operands[0] as usize;
            },
            Opcode::OP_END => {
                // if has one operand, it points to while
                // if has no operands, it ends an if => falthrough
                if ins.operands.len() == 1 {
                    ip = ins.operands[0] as usize;
                }
            },
            Opcode::OP_WHILE => { },
            Opcode::OP_DO => {
                let a = stack.pop().unwrap();
                if a == 0 {
                    ip = ins.operands[0] as usize;
                }
            }
        }
        // print!("{} ", ip);
        // _dump_stack(&stack);
        ip += 1;
    }
}

fn compile(program : &Vec<Instruction>, exec_file: &str, run_prog : bool) {
    codegen(program, &exec_file);
    let status = build(&exec_file);
    if status == 1 {
        _dump_bytecode(&program);
        process::exit(1);
    }
    if run_prog {
        execute(&exec_file);
    }
}

fn codegen(program: &Vec<Instruction>, exec_file : &str) {
    let asm_filename = exec_file.to_string() + ".asm";
    let mut asm_file = File::create(asm_filename)
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
    writeln!(&mut asm_file, "    mul     r10").unwrap();
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
                writeln!(&mut asm_file, ".addr_{}: ;; OP_PUSH", ins.ip).unwrap();
                writeln!(&mut asm_file, "    push {}", ins.operands[0]).unwrap();
            },
            Opcode::OP_ADD => {
                writeln!(&mut asm_file, ".addr_{}: ;; OP_ADD", ins.ip).unwrap();
                writeln!(&mut asm_file, "    pop rax").unwrap();
                writeln!(&mut asm_file, "    pop rbx").unwrap();
                writeln!(&mut asm_file, "    add rax, rbx").unwrap();
                writeln!(&mut asm_file, "    push rax").unwrap();
            },
            Opcode::OP_SUB => {
                writeln!(&mut asm_file, ".addr_{}: ;; OP_SUB", ins.ip).unwrap();
                writeln!(&mut asm_file, "    pop rax").unwrap();
                writeln!(&mut asm_file, "    pop rbx").unwrap();
                writeln!(&mut asm_file, "    sub rbx, rax").unwrap();
                writeln!(&mut asm_file, "    push rbx").unwrap();
            },
            Opcode::OP_MUL => {
                writeln!(&mut asm_file, ".addr_{}: ;; OP_MUL", ins.ip).unwrap();
                writeln!(&mut asm_file, "    pop rax").unwrap();
                writeln!(&mut asm_file, "    pop rbx").unwrap();
                writeln!(&mut asm_file, "    mul rbx").unwrap();
                writeln!(&mut asm_file, "    push rax").unwrap();
            },
            Opcode::OP_DIV => {
                //FIXME: not working
                writeln!(&mut asm_file, ".addr_{}: ;; OP_DIV", ins.ip).unwrap();
                writeln!(&mut asm_file, "    xor rdx, rdx").unwrap();
                writeln!(&mut asm_file, "    pop rbx").unwrap();
                writeln!(&mut asm_file, "    pop rax").unwrap();
                writeln!(&mut asm_file, "    div rbx").unwrap();
                writeln!(&mut asm_file, "    push rax").unwrap();
                writeln!(&mut asm_file, "    push rdx").unwrap();
            },
            Opcode::OP_NOT => {
                writeln!(&mut asm_file, ".addr_{}: ;; OP_NOT", ins.ip).unwrap();
                writeln!(&mut asm_file, "    pop rax").unwrap();
                writeln!(&mut asm_file, "    not rax").unwrap();
                writeln!(&mut asm_file, "    push rax").unwrap();
            },
            Opcode::OP_EQ => {
                writeln!(&mut asm_file, ".addr_{}: ;; OP_EQ", ins.ip).unwrap();
                writeln!(&mut asm_file, "    mov rcx, 0").unwrap();
                writeln!(&mut asm_file, "    mov rdx, 1").unwrap();
                writeln!(&mut asm_file, "    pop rax").unwrap();
                writeln!(&mut asm_file, "    pop rbx").unwrap();
                writeln!(&mut asm_file, "    cmp rax, rbx").unwrap();
                writeln!(&mut asm_file, "    cmove rcx, rdx").unwrap();
                writeln!(&mut asm_file, "    push rcx").unwrap();
            },
            Opcode::OP_NE => {
                writeln!(&mut asm_file, ".addr_{}: ;; OP_NE", ins.ip).unwrap();
                writeln!(&mut asm_file, "    mov rcx, 0").unwrap();
                writeln!(&mut asm_file, "    mov rdx, 1").unwrap();
                writeln!(&mut asm_file, "    pop rax").unwrap();
                writeln!(&mut asm_file, "    pop rbx").unwrap();
                writeln!(&mut asm_file, "    cmp rax, rbx").unwrap();
                writeln!(&mut asm_file, "    cmovne rcx, rdx").unwrap();
                writeln!(&mut asm_file, "    push rcx").unwrap();
            },
            Opcode::OP_GT => {
                writeln!(&mut asm_file, ".addr_{}: ;; OP_GT", ins.ip).unwrap();
                writeln!(&mut asm_file, "    mov rcx, 0").unwrap();
                writeln!(&mut asm_file, "    mov rdx, 1").unwrap();
                writeln!(&mut asm_file, "    pop rbx").unwrap();
                writeln!(&mut asm_file, "    pop rax").unwrap();
                writeln!(&mut asm_file, "    cmp rax, rbx").unwrap();
                writeln!(&mut asm_file, "    cmovg rcx, rdx").unwrap();
                writeln!(&mut asm_file, "    push rcx").unwrap();
            },
            Opcode::OP_GE => {
                writeln!(&mut asm_file, ".addr_{}: ;; OP_GE", ins.ip).unwrap();
                writeln!(&mut asm_file, "    mov rcx, 0").unwrap();
                writeln!(&mut asm_file, "    mov rdx, 1").unwrap();
                writeln!(&mut asm_file, "    pop rbx").unwrap();
                writeln!(&mut asm_file, "    pop rax").unwrap();
                writeln!(&mut asm_file, "    cmp rax, rbx").unwrap();
                writeln!(&mut asm_file, "    cmovge rcx, rdx").unwrap();
                writeln!(&mut asm_file, "    push rcx").unwrap();
            },
            Opcode::OP_LT => {
                writeln!(&mut asm_file, ".addr_{}: ;; OP_LT", ins.ip).unwrap();
                writeln!(&mut asm_file, "    mov rcx, 0").unwrap();
                writeln!(&mut asm_file, "    mov rdx, 1").unwrap();
                writeln!(&mut asm_file, "    pop rbx").unwrap();
                writeln!(&mut asm_file, "    pop rax").unwrap();
                writeln!(&mut asm_file, "    cmp rax, rbx").unwrap();
                writeln!(&mut asm_file, "    cmovl rcx, rdx").unwrap();
                writeln!(&mut asm_file, "    push rcx").unwrap();
            },
            Opcode::OP_LE => {
                writeln!(&mut asm_file, ".addr_{}: ;; OP_LE", ins.ip).unwrap();
                writeln!(&mut asm_file, "    mov rcx, 0").unwrap();
                writeln!(&mut asm_file, "    mov rdx, 1").unwrap();
                writeln!(&mut asm_file, "    pop rbx").unwrap();
                writeln!(&mut asm_file, "    pop rax").unwrap();
                writeln!(&mut asm_file, "    cmp rax, rbx").unwrap();
                writeln!(&mut asm_file, "    cmovle rcx, rdx").unwrap();
                writeln!(&mut asm_file, "    push rcx").unwrap();
            },
            Opcode::OP_DUP => {
                writeln!(&mut asm_file, ".addr_{}: ;; OP_DUP", ins.ip).unwrap();
                writeln!(&mut asm_file, "    pop rax").unwrap();
                writeln!(&mut asm_file, "    push rax").unwrap();
                writeln!(&mut asm_file, "    push rax").unwrap();
            }
            Opcode::OP_DUMP => {
                writeln!(&mut asm_file, ".addr_{}: ;; OP_DUMP", ins.ip).unwrap();
                writeln!(&mut asm_file, "    pop rdi").unwrap();
                writeln!(&mut asm_file, "    call dump").unwrap();
            },
            Opcode::OP_IF => {
                writeln!(&mut asm_file, ".addr_{}: ;; OP_IF", ins.ip).unwrap();
                writeln!(&mut asm_file, "    pop rax").unwrap();
                writeln!(&mut asm_file, "    test rax, rax").unwrap();
                writeln!(&mut asm_file, "    jz .addr_{}", ins.operands[0]+1).unwrap();
            },
            Opcode::OP_ELSE => {
                writeln!(&mut asm_file, ".addr_{}: ;; OP_ELSE", ins.ip).unwrap();
                writeln!(&mut asm_file, "    jmp .addr_{}", ins.operands[0]+1).unwrap();
            },
            Opcode::OP_END => {
                if ins.operands.len() == 0 {
                    // no operands means it ends an if => flalthrough
                    continue;
                } else {
                    // points back to while
                    writeln!(&mut asm_file, ".addr_{}: ;; OP_END", ins.ip).unwrap();
                    writeln!(&mut asm_file, "    jmp .addr_{}", ins.operands[0]).unwrap();
                }
            },
            Opcode::OP_WHILE => {
                writeln!(&mut asm_file, ".addr_{}: ;; OP_WHILE", ins.ip).unwrap();
            },
            Opcode::OP_DO => {
                writeln!(&mut asm_file, ".addr_{}: ;; OP_DO", ins.ip).unwrap();
                writeln!(&mut asm_file, "    pop rax").unwrap();
                writeln!(&mut asm_file, "    test rax, rax").unwrap();
                writeln!(&mut asm_file, "    jz .addr_{}", ins.operands[0]+1).unwrap();
            }
        }
    }
    writeln!(&mut asm_file, ".end:").unwrap();
    writeln!(&mut asm_file, "    mov rax, SYS_EXIT").unwrap();
    writeln!(&mut asm_file, "    mov rdi, 0").unwrap();
    writeln!(&mut asm_file, "    syscall").unwrap();
    writeln!(&mut asm_file, "    ret").unwrap();
}

fn build(exec_file : &str) -> usize{
    let asm_filename = exec_file.to_string() + ".asm";
    let compiler_status = Command::new("nasm")
        .args(["-felf64", asm_filename.as_str()])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .unwrap();

    match compiler_status.code() {
        Some(0) => { },
        Some(_) => { return 1; }
        None => { return 1; }
    }

    let obj_filename = exec_file.to_string() + ".o";
    let linker_status = Command::new("ld")
        .args(["-o", exec_file, obj_filename.as_str()])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .unwrap();

    match linker_status.code() {
        Some(0) => { },
        Some(_) => { return 1; }
        None => { return 1; }
    }

    0
}

fn execute(exec_file : &str) {
    let mut exec_filename  = String::from(exec_file);
    exec_filename.insert_str(0, "./");
    let _program_output = Command::new(exec_filename)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .unwrap();
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn parse_push() {
        let tokens : Vec<Token> = vec![Token::new(String::from("2"), 0, 0)];
        let program = parser("", &tokens);
        assert_eq!(program[0].opcode, Opcode::OP_PUSH);
    }
    #[test]
    
    fn parse_add() {
        let tokens : Vec<Token> = vec![Token::new(String::from("+"), 0, 0)];
        let program = parser("", &tokens);
        assert_eq!(program[0].opcode, Opcode::OP_ADD);
    }

    #[test]
    fn compile_generates_executable() {
        let source_file = "tests/arithmetic.rorth";
        let tokens = lexer(&source_file);
        let program = parser(&source_file, &tokens);
        compile(&program, "test_compile_generates_executable", false);
        assert_eq!(std::path::Path::new("./test_compile_generates_executable.asm").exists(), true);
        assert_eq!(std::path::Path::new("./test_compile_generates_executable.o").exists(), true);
        assert_eq!(std::path::Path::new("./test_compile_generates_executable").exists(), true);
        fs::remove_file("./test_compile_generates_executable.asm").unwrap();
        fs::remove_file("./test_compile_generates_executable.o").unwrap();
        fs::remove_file("./test_compile_generates_executable").unwrap();
    }

    #[test]
    fn interpret_arithmetic() {
        let source_file = "tests/arithmetic.rorth";
        let tokens = lexer(&source_file);
        let program = parser(&source_file, &tokens);
        let mut stdout = Vec::new();
        interpret(&program, &mut stdout);
        assert_eq!(stdout, b"69\n420\n4\n5\n");
    }

    #[test]
    fn interpret_comparisons() {
        let source_file = "tests/comparisons.rorth";
        let tokens = lexer(&source_file);
        let program = parser(&source_file, &tokens);
        let mut stdout = Vec::new();
        interpret(&program, &mut stdout);
        assert_eq!(stdout, b"1\n0\n0\n1\n1\n0\n0\n1\n");
    }

    #[test]
    fn interpret_ifs() {
        let source_file = "tests/if.rorth";
        let tokens = lexer(&source_file);
        let program = parser(&source_file, &tokens);
        let mut stdout = Vec::new();
        interpret(&program, &mut stdout);
        assert_eq!(stdout, b"1\n42\n42\n0\n42\n");
    }

    #[test]
    fn interpret_whiles() {
        let source_file = "tests/while.rorth";
        let tokens = lexer(&source_file);
        let program = parser(&source_file, &tokens);
        let mut stdout = Vec::new();
        interpret(&program, &mut stdout);
        assert_eq!(stdout, b"10\n9\n8\n7\n6\n5\n4\n3\n2\n1\n420\n");
    }

    #[test]
    fn compile_comparisons() {
        let source_file = "tests/comparisons.rorth";
        let tokens = lexer(&source_file);
        let program = parser(&source_file, &tokens);
        compile(&program, "test_compile_comparisons", false);
        let exec_output = Command::new("./test_compile_comparisons")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Expected a 0 return code");
        assert_eq!(exec_output.stdout, b"1\n0\n0\n1\n1\n0\n0\n1\n");
        fs::remove_file("./test_compile_comparisons.asm").unwrap();
        fs::remove_file("./test_compile_comparisons.o").unwrap();
        fs::remove_file("./test_compile_comparisons").unwrap();
    }
    #[test]
    fn compile_ifs() {
        let source_file = "tests/if.rorth";
        let tokens = lexer(&source_file);
        let program = parser(&source_file, &tokens);
        compile(&program, "test_compile_ifs", false);
        let exec_output = Command::new("./test_compile_ifs")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Expected a 0 return code");
        assert_eq!(exec_output.stdout, b"1\n42\n42\n0\n42\n");
        fs::remove_file("./test_compile_ifs.asm").unwrap();
        fs::remove_file("./test_compile_ifs.o").unwrap();
        fs::remove_file("./test_compile_ifs").unwrap();
    }
}