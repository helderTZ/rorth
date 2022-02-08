use std::env;
use std::fs::File;
use std::io::Write;
use std::process;
use std::process::{Command, Stdio};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[allow(non_camel_case_types)]
enum Opcode {
    OP_PUSH,
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
        compile(source_file.as_str(), run_prog);
    }
}

fn interpret(file : &str) {
    let program : Vec<Instruction> = vec![
        Instruction::new(Opcode::OP_PUSH, vec![34]),
        Instruction::new(Opcode::OP_PUSH, vec![35]),
        Instruction::new(Opcode::OP_ADD, vec![]),
        Instruction::new(Opcode::OP_DUMP, vec![]),
        Instruction::new(Opcode::OP_PUSH, vec![430]),
        Instruction::new(Opcode::OP_PUSH, vec![10]),
        Instruction::new(Opcode::OP_MINUS, vec![]),
        Instruction::new(Opcode::OP_DUMP, vec![]),
    ];

    let mut stack : Vec<i64> = Vec::new();
    for ins in program {
        match ins.opcode {
            Opcode::OP_PUSH => {
                stack.push(ins.operands[0]);
            },
            Opcode::OP_ADD => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a+b);
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

fn compile(file: &str, run_prog : bool) {
    let program : Vec<Instruction> = vec![
        Instruction::new(Opcode::OP_PUSH, vec![34]),
        Instruction::new(Opcode::OP_PUSH, vec![35]),
        Instruction::new(Opcode::OP_ADD, vec![]),
        Instruction::new(Opcode::OP_DUMP, vec![]),
        Instruction::new(Opcode::OP_PUSH, vec![430]),
        Instruction::new(Opcode::OP_PUSH, vec![10]),
        Instruction::new(Opcode::OP_MINUS, vec![]),
        Instruction::new(Opcode::OP_DUMP, vec![]),
    ];

    codegen(program);
    build();

    if run_prog {
        execute();
    }
}

fn codegen(program: Vec<Instruction>) {
    let mut asm_file = File::create("out.asm")
        .expect("Could not open file");
    writeln!(&mut asm_file, "%define SYS_EXIT 60").unwrap();
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
    writeln!(&mut asm_file, "    mov     rax, 1").unwrap();
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
            Opcode::OP_MINUS => {
                writeln!(&mut asm_file, "    pop rax").unwrap();
                writeln!(&mut asm_file, "    pop rbx").unwrap();
                writeln!(&mut asm_file, "    sub rbx, rax").unwrap();
                writeln!(&mut asm_file, "    push rbx").unwrap();
            },
            Opcode::OP_DUMP => {
                writeln!(&mut asm_file, "    pop rdi").unwrap();
                writeln!(&mut asm_file, "    call dump").unwrap();
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
    println!("compiler output: {}", String::from_utf8(compiler_output.stdout).unwrap());
    println!("compiler stderr: {}", String::from_utf8(compiler_output.stderr).unwrap());

    let linker_output = Command::new("ld")
        .args(["-o", "out", "out.o"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    println!("linker output: {}", String::from_utf8(linker_output.stdout).unwrap());
    println!("linker stderr: {}", String::from_utf8(linker_output.stderr).unwrap());
}

fn execute() {
    let program_output = Command::new("out")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    println!("program output: {}", String::from_utf8(program_output.stdout).unwrap());
    println!("program stderr: {}", String::from_utf8(program_output.stderr).unwrap());
}