use std::collections::HashSet;

use sexp::Atom::*;
use sexp::Sexp::*;
use sexp::*;

use crate::types::*;

use ZeroOp::*;
use UnitaryOp::*;
use BinaryOp::*;
use Expr::*;

fn parse_bind(vec: &Vec<Sexp>) -> Vec<Binding> {
    let mut array = Vec::new();
    for binding in &vec[..] {
        if let List(inner_vec) = binding { if let [Atom(S(s)), e] = &inner_vec[..] {
            array.push(Binding(parse_identifier(s), Box::new(parse_expr(e))));
            continue
        } }
        panic!("Invalid - parse error")
    }
    array
}

fn parse_identifier(s: &str) -> String {
    let keywords = HashSet::from(["true", "false", "input", "let", "set!", "if", "block", "loop", "break", "add1", "sub1", "isnum", "isbool", "array", "link", "fill", "deref"]);
    (!keywords.contains(s)).then(||0).expect("Invalid - parse error - keyword or function name used as identifier!");
    String::from(s)
}

fn parse_def(sexp: &Sexp, reserved: &mut HashSet<String>) -> Expr {
    if let List(vec_def) = sexp { match &vec_def[..] {
        [Atom(S(op)), List(strings), e] if op == "fun" => {
            let names: Vec::<String> = strings.iter().map(|s| if let Atom(S(name)) = s {return parse_identifier(name)}
                else {panic!("Invalid - parse error - expected simple string in def!")}).collect();
            names.first().expect("Invalid - parse error - func def expected name!");
            reserved.insert(names[0].clone()).then(||0).expect("Invalid - parse error - name is already in use!");
            return FuncDef(names[0].clone(), names[1..].to_vec(), Box::new(parse_expr(e)))
        },
        _ => (),
    } }
    panic!("Invalid - parse error - failed to match def!");
}

fn parse_expr(sexp: &Sexp) -> Expr {
    match sexp {
        Atom(item) => match item {
            I(n) => if *n <= 4611686018427387903 && *n >= -4611686018427387904 {EZeroOp(Number(2*i64::try_from(*n).expect("Invalid - parse error - number not valid or overflow!")))}
                                else {panic!("Invalid - parse error - number not valid or overflow!")},
            S(op) if op == "true" => EZeroOp(OpTrue),
            S(op) if op == "false" => EZeroOp(OpFalse),
            S(op) if op == "input" => EZeroOp(Input),
            S(s) => EZeroOp(Identifier(s.clone())),
            F(_) => panic!("Floats not implemented!"),
        },
        List(vec) => {
            match &vec[..] {
                [Atom(S(op)), e] if op == "add1" => EUnitaryOp(Add1, Box::new(parse_expr(e))),
                [Atom(S(op)), e] if op == "sub1" => EUnitaryOp(Sub1, Box::new(parse_expr(e))),
                [Atom(S(op)), e] if op == "isnum" => EUnitaryOp(IsNum, Box::new(parse_expr(e))),
                [Atom(S(op)), e] if op == "isbool" => EUnitaryOp(IsBool, Box::new(parse_expr(e))),
                [Atom(S(op)), e] if op == "loop" => EUnitaryOp(Loop, Box::new(parse_expr(e))),
                [Atom(S(op)), e] if op == "break" => EUnitaryOp(Break, Box::new(parse_expr(e))),
                [Atom(S(op)), e1, e2] if op == "+" => EBinaryOp(Plus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
                [Atom(S(op)), e1, e2] if op == "-" => EBinaryOp(Minus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
                [Atom(S(op)), e1, e2] if op == "*" => EBinaryOp(Times, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
                [Atom(S(op)), e1, e2] if op == ">" => EBinaryOp(Greater, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
                [Atom(S(op)), e1, e2] if op == ">=" => EBinaryOp(GreaterEqual, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
                [Atom(S(op)), e1, e2] if op == "<" => EBinaryOp(Less, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
                [Atom(S(op)), e1, e2] if op == "<=" => EBinaryOp(LessEqual, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
                [Atom(S(op)), e1, e2] if op == "=" => EBinaryOp(Equal, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
                [Atom(S(op)), List(vec_let), e] if op == "let" && (!vec_let.is_empty()).then(||true).expect("Invalid - Need bindings in let!") =>
                    Let(parse_bind(vec_let), Box::new(parse_expr(e))),
                [Atom(S(op)), Atom(S(var)), e] if op == "set!" => Set(Binding(parse_identifier(var.as_str()), Box::new(parse_expr(e)))),
                [Atom(S(op)), e1, e2, e3] if op == "if" => If(Box::new(parse_expr(e1)), Box::new(parse_expr(e2)), Box::new(parse_expr(e3))),
                [Atom(S(op)), vec_block @ ..] if op == "block" && (!vec_block.is_empty()).then(||true).expect("Invalid - block must have subexpressions!") =>
                    Block(vec_block.iter().map(|e| parse_expr(e)).collect()),
                [Atom(S(name)), vec_call @ ..] => FuncCall(name.clone(), vec_call.iter().map(|arg| Box::new(parse_expr(arg))).collect()),
                _ => panic!("Invalid - parse error - unexpected structure! {:?}", vec),
            }
        },
    }
}

pub fn parse_program(sexp: &Sexp) -> Expr {
    let mut reserved = HashSet::<String>::new();
    if let List(items) = sexp { match &items[..] {
        [defs @ .., e] => return Program(defs.iter().map(|def| parse_def(def, &mut reserved)).collect(), Box::new(parse_expr(e))),
        _ => panic!("Invalid - parse error - Program requires main expression!")
    } }
    panic!("Invalid - parse error - malformed program!");
}