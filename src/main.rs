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
    in_contents = format!("({})", in_contents);

    let sexp = parse(&in_contents).expect("Invalid - failed to parse sexp");
    let expr = parse_program(&sexp);

    let result = compile_program(&expr);

    let asm_program = format!(
        "
section .text

extern snek_error, raise_error
extern failed_setup, no_more_mem, type_error, err_bad_access, expect_bool, expect_numeric, overflow
extern def_print
extern heap_setup
extern def_array, def_link, def_link_from, def_link_to
extern alloc, def_fill, def_deref, def_index

global our_code_starts_here

{}
",
        result
    );

    let mut out_file = File::create(out_name)?;
    out_file.write_all(asm_program.as_bytes())?;

    Ok(())
}