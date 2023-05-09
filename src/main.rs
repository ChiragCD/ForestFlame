use std::env;
use std::fs::File;
use std::io::prelude::*;

use diamondback::parser::*;
use diamondback::compiler::*;

use sexp::*;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let in_name = &args[1];
    let out_name = &args[2];

    let mut in_file = File::open(in_name)?;
    let mut in_contents = String::new();
    in_file.read_to_string(&mut in_contents)?;

    let sexp = parse(&in_contents).expect("Invalid - failed to parse sexp");
    let expr = parse_expr(&sexp);

    let result = compile_expr(&expr);

    let asm_program = format!(
        "
section .text
extern snek_error
global our_code_starts_here

expect_bool:
mov rdi, 5
jmp snek_error

expect_numeric:
mov rdi, 6
jmp snek_error

overflow:
mov rdi, 7
jmp snek_error

our_code_starts_here:
{}
ret
",
        result
    );

    let mut out_file = File::create(out_name)?;
    out_file.write_all(asm_program.as_bytes())?;

    Ok(())
}