use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::cmp::*;

use crate::types::*;

use ZeroOp::*;
use UnitaryOp::*;
use BinaryOp::*;
use Expr::*;
use Instr::*;
use Val::*;
use Register::*;

const STACK_BASE: i64 = 2;

#[derive(Debug, Clone)]
struct Namespace {
    func: String,
    h: HashMap<String, Val>,
    break_label: String,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
enum Type { Num, Bool, Unknown, }
use Type::*;

struct State {
    defs: HashMap<String, usize>,
    types: HashMap<Val, Type>,
}

thread_local!(static STATE: Rc<RefCell<State>> = Rc::new(RefCell::new(State{types: HashMap::new(), defs: HashMap::new(), })));

fn generate_id(uid: &mut i64) -> i64 {
    *uid += 1;
    *uid
}

fn push(v: &mut Vec<Instr>, reg: Register, stack: &mut i64) -> Val {
    let loc = RegOffset(RSP, *stack);
    v.push(IMov(loc.clone(), Reg(reg)));
    set_type(&loc, get_type(&Reg(reg)));
    *stack += 1;
    loc
}

fn load(v: &mut Vec<Instr>, reg: Register, loc: &Val) {
    v.push(IMov(Reg(reg), loc.clone()));
    set_type(&Reg(reg), get_type(loc));
}

fn fail_overflow(v: &mut Vec<Instr>) {
    v.push(Jo(Label(String::from("overflow"))));
}

fn get_type(val: &Val) -> Type {
    return Unknown;
    if matches!(*val, Imm(_)) {return Num};
    if matches!(*val, ValFalse) {return Bool};
    if matches!(*val, ValTrue) {return Bool};
    STATE.with(|x| {
        let state = x.borrow_mut();
        *state.types.get(&val.clone()).unwrap_or_else(|| &Unknown)
    })
}

fn set_type(val: &Val, datatype: Type) {
    STATE.with(|x| {
        let mut state = x.borrow_mut();
        state.types.insert(val.clone(), datatype);
    })
}

fn matches_type(type1: Type, type2: Type) -> bool {
    if type1 == Unknown || type2 == Unknown {return false};
    type1 == type2
}

fn expect_number(v: &mut Vec<Instr>, reg: Register) {
    if matches_type(get_type(&Reg(reg)), Num) {return};
    v.push(Test(Reg(reg), Imm(1)));
    v.push(Jnz(Label(format!("expect_numeric"))));
    set_type(&Reg(reg), Num);
}

fn compile_zero_op(op: &ZeroOp, v: &mut Vec<Instr>, namespace: &mut Namespace) {
    match op {
        Number(n) => load(v, RAX, &Imm(*n)),
        OpTrue => load(v, RAX, &ValTrue),
        OpFalse => load(v, RAX, &ValFalse),
        Input => (namespace.func == "main").then(|| load(v, RAX, &Reg(RDI))).expect("Invalid - cannot use input outside main"),
        Identifier(s) => load(v, RAX, &namespace_get(namespace, s)),
    }
}

fn compile_if(expr1: &Expr, expr2: &Expr, expr3: &Expr, v: &mut Vec<Instr>, stack: i64, uid: &mut i64, namespace: &mut Namespace) {
    compile(expr1, v, stack, uid, namespace);
    let id = generate_id(uid);
    let label_false = format!("if_false_{}", id);
    let label_end = format!("end_if_{}", id);
    v.push(Cmp(Reg(RAX), ValFalse));                // else assume ValTrue or number
    v.push(Je(Label(label_false.clone())));
    compile(expr2, v, stack, uid, namespace);
    v.push(Jmp(Label(label_end.clone())));
    v.push(ILabel(Label(format!("{}:", label_false.clone()))));
    compile(expr3, v, stack, uid, namespace);
    v.push(Jmp(Label(label_end.clone())));
    v.push(ILabel(Label(format!("{}:", label_end.clone()))));
}

fn compile_loop(e: &Expr, v: &mut Vec<Instr>, stack: i64, uid: &mut i64, namespace: &mut Namespace) {
    let label = format!("loop_{}", generate_id(uid));
    let mut inner_namespace = namespace.clone();
    inner_namespace.break_label = label.clone();
    v.push(ILabel(Label(format!("{}:", label.as_str()))));
    compile(e, v, stack, uid, &mut inner_namespace);
    v.push(Jmp(Label(label.clone())));
    v.push(ILabel(Label(format!("end_{}:", label.as_str()))));
}

fn compile_unitary_op(op: &UnitaryOp, e: &Expr, v: &mut Vec<Instr>, stack: i64, uid: &mut i64, namespace: &mut Namespace) {
    matches!(op, Loop).then(|| compile_loop(e, v, stack, uid, namespace));
    (!matches!(op, Loop)).then(|| compile(e, v, stack, uid, namespace));
    match op {
        Add1 => {
            expect_number(v, RAX);
            v.push(IAdd(Reg(RAX), Imm(2)));
            fail_overflow(v);
        }
        Sub1 => {
            expect_number(v, RAX);
            v.push(ISub(Reg(RAX), Imm(2)));
            fail_overflow(v);
        }
        IsNum => {
            v.push(Not(Reg(RAX)));
            v.push(And(Reg(RAX), Imm(1)));
            v.push(Shl(Reg(RAX), Imm(1)));
            v.push(Inc(Reg(RAX)));
            set_type(&Reg(RAX), Bool);
        },
        IsBool => {
            v.push(And(Reg(RAX), Imm(1)));
            v.push(Shl(Reg(RAX), Imm(1)));
            v.push(Inc(Reg(RAX)));
            set_type(&Reg(RAX), Bool);
        },
        Break => {
            (namespace.break_label != "").then(||0).expect("Invalid - parse error - break outside a loop body!");
            v.push(Jmp(Label(format!("end_{}", namespace.break_label.as_str()))));
        },
        Loop => (),
    }
}

fn compile_binary_op_to_arithmetic(op: &BinaryOp, e1: &Expr, e2: &Expr, v: &mut Vec<Instr>, mut stack: i64, uid: &mut i64, namespace: &mut Namespace) {
    compile(e2, v, stack, uid, namespace);
    let loc = push(v, RAX, &mut stack);
    compile(e1, v, stack, uid, namespace);
    load(v, RBX, &loc);
    expect_number(v, RAX);
    expect_number(v, RBX);
    match op {
        Plus => {
            v.push(IAdd(Reg(RAX), Reg(RBX)));
            fail_overflow(v);
        }
        Minus => {
            v.push(ISub(Reg(RAX), Reg(RBX)));
            fail_overflow(v);
        }
        Times => {
            v.push(Sar(Reg(RAX), Imm(1)));
            v.push(Sar(Reg(RBX), Imm(1)));
            v.push(IMul(Reg(RAX), Reg(RBX)));
            fail_overflow(v);
            v.push(Sal(Reg(RAX), Imm(1)));
            fail_overflow(v);
        }
        _ => panic!("Compile to arithmetic binary op called for non binary to arithmetic!"),
    }
}

fn compile_binary_op_arithmetic_to_boolean(op: &BinaryOp, e1: &Expr, e2: &Expr, v: &mut Vec<Instr>, mut stack: i64, uid: &mut i64, namespace: &mut Namespace) {
    compile(e2, v, stack, uid, namespace);
    let loc = push(v, RAX, &mut stack);
    compile(e1, v, stack, uid, namespace);
    load(v, RBX, &loc);
    expect_number(v, RAX);
    expect_number(v, RBX);
    v.push(Cmp(Reg(RAX), Reg(RBX)));
    load(v, RAX, &ValFalse);
    load(v, RBX, &ValTrue);
    match op {
        Greater => v.push(Cmovg(Reg(RAX), Reg(RBX))),
        GreaterEqual => v.push(Cmovge(Reg(RAX), Reg(RBX))),
        Less => v.push(Cmovl(Reg(RAX), Reg(RBX))),
        LessEqual => v.push(Cmovle(Reg(RAX), Reg(RBX))),
        _ => panic!("Bad call to compile arithmetic to bool binary!"),
    }
}

fn compile_equal(e1: &Expr, e2: &Expr, v: &mut Vec<Instr>, mut stack: i64, uid: &mut i64, namespace: &mut Namespace) {
    compile(e2, v, stack, uid, namespace);
    let loc = push(v, RAX, &mut stack);
    compile(e1, v, stack, uid, namespace);
    load(v, RBX, &loc);
    let type_rax = get_type(&Reg(RAX));
    let type_rbx = get_type(&Reg(RBX));
    v.push(Xor(Reg(RBX), Reg(RAX)));
    set_type(&(Reg(RBX)), Unknown);
    load(v, RBP, &ValTrue);
    load(v, RAX, &ValFalse);
    v.push(Cmovz(Reg(RAX), Reg(RBP)));
    if matches_type(type_rax, type_rbx) {return};
    set_type(&(Reg(RBX)), Unknown);
    expect_number(v, RBX);
}

fn compile_binary_op(op: &BinaryOp, e1: &Expr, e2: &Expr, v: &mut Vec<Instr>, stack: i64, uid: &mut i64, namespace: &mut Namespace) {
    match op {
        Equal => compile_equal(e1, e2, v, stack, uid, namespace),
        Greater => compile_binary_op_arithmetic_to_boolean(op, e1, e2, v, stack, uid, namespace),
        GreaterEqual => compile_binary_op_arithmetic_to_boolean(op, e1, e2, v, stack, uid, namespace),
        Less => compile_binary_op_arithmetic_to_boolean(op, e1, e2, v, stack, uid, namespace),
        LessEqual => compile_binary_op_arithmetic_to_boolean(op, e1, e2, v, stack, uid, namespace),
        Plus => compile_binary_op_to_arithmetic(op, e1, e2, v, stack, uid, namespace),
        Minus => compile_binary_op_to_arithmetic(op, e1, e2, v, stack, uid, namespace),
        Times => compile_binary_op_to_arithmetic(op, e1, e2, v, stack, uid, namespace),
    }
}

fn namespace_get(namespace: &Namespace, s: &String) -> Val {
    namespace.h.get(s).expect(format!("Unbound variable identifier {}", s).as_str()).clone()
}

fn namespace_add(namespace: &mut Namespace, s: &String, loc: &Val, must_not_exist: bool) {
    let exists = namespace.h.insert(s.clone(), loc.clone()).is_some();
    (!(must_not_exist && exists)).then(||0).expect("Duplicate binding");
}

fn compile_binding(binding: &Binding, v: &mut Vec<Instr>, stack: &mut i64, uid: &mut i64, namespace: &mut Namespace, must_exist: bool, must_not_exist: bool) {
    let Binding(s, e) = binding;
    compile(e, v, *stack, uid, namespace);
    if must_exist {
        let location = namespace_get(namespace, s);
        v.push(IMov(location.clone(), Reg(RAX)));
        set_type(&location, get_type(&Reg(RAX)));
    } else {
        let location = push(v, RAX, stack);
        namespace_add(namespace, s, &location, must_not_exist);
    }
}

fn compile_let(e: &Expr, vec: &Vec<Binding>, v: &mut Vec<Instr>, mut stack: i64, uid: &mut i64, namespace: &mut Namespace) {
    let mut inner_namespace = namespace.clone();
    _ = vec.iter().map(|binding| compile_binding(binding, v, &mut stack, uid, &mut inner_namespace, false, false)).collect::<Vec<_>>();
    compile(e, v, stack, uid, &mut inner_namespace);
}

fn compile_def(e: &Expr, v: &mut Vec<Instr>, uid: &mut i64) {
    let (name, vec_params, action) = match e {
        FuncDef(name, vec_params, action) => (name, vec_params, action),
        _ => panic!("Invalid - Expected a definition!"),
    };
    STATE.with(|x| {
        let mut state = x.borrow_mut();
        state.defs.insert(name.clone(), vec_params.len()).is_none().then(||0).expect("Invalid - name is already in use!");
    });
    v.push(ILabel(Label(format!("def_{}:", name))));
    let mut namespace = Namespace{h: HashMap::new(), break_label: String::from(""), func: String::from(name)};
    let locations = vec![RDI, RSI, RDX, RCX, R8, R9];
    for i in 0..(min(6, vec_params.len())) {namespace_add(&mut namespace, &vec_params[i], &Reg(locations[i]), true)};
    for i in 6..vec_params.len() {namespace_add(&mut namespace, &vec_params[i], &RegOffset(RSP, 8*(5-(i)) as i64), true)};
    compile(action, v, STACK_BASE, uid, &mut namespace);
    v.push(Ret);
}

fn compile_call(name: &String, args: &Vec<Box<Expr>>, v: &mut Vec<Instr>, stack: &mut i64, uid: &mut i64, namespace: &mut Namespace) {
    let locations = vec![RDI, RSI, RDX, RCX, R8, R9];
    for i in 0..(min(6, args.len())) {
        compile(&args[i], v, *stack, uid, namespace);
        load(v, locations[i], &Reg(RAX));
    };
    for i in (5..(max(0, args.len() as i64 - 1))).rev() {
        compile(&args[i as usize], v, *stack, uid, namespace);
        _ = push(v, RAX, stack);
    };
    STATE.with(|x| {
        let state = x.borrow_mut();
        let length = *state.defs.get(name).expect("Invalid - function not found!");
        (length == args.len()).then(||0).expect("Invalid - call has wrong number of args!");
    });
    v.push(IAdd(Reg(RSP), Imm(8**stack)));
    v.push(Call(Label(format!("def_{}", name))));
    v.push(ISub(Reg(RSP), Imm(8**stack)));
}

fn compile(e: &Expr, v: &mut Vec<Instr>, stack: i64, uid: &mut i64, namespace: &mut Namespace) {
    match e {
        EZeroOp(op) => compile_zero_op(op, v, namespace),
        EUnitaryOp(op, expr) => compile_unitary_op(op, expr, v, stack, uid, namespace),
        EBinaryOp(op, expr1, expr2) => compile_binary_op(op, expr1, expr2, v, stack, uid, namespace),
        Let(vec, e) => compile_let(e, vec, v, stack, uid, namespace),
        Set(binding) => compile_binding(binding, v, &mut stack.clone(), uid, namespace, true, false),
        If(expr1, expr2, expr3) => compile_if(expr1, expr2, expr3, v, stack, uid, namespace),
        Block(vec) => vec.iter().map(|e| compile(e, v, stack, uid, namespace)).collect(),
        FuncDef(_, _, _) => panic!("Invalid - unexpected function def in non global scope!"),
        FuncCall(name, args) => compile_call(name, args, v, &mut stack.clone(), uid, namespace),
        Program(defs, e) => {
            _ = defs.iter().map(|def| compile_def(def, v, uid)).collect::<Vec<_>>();
            v.push(ILabel(Label(String::from("our_code_starts_here:"))));
            compile(e, v, stack, uid, namespace);
            v.push(Ret);
        }
    }
}

fn read_val(v: Val) -> String {
    match v {
        Reg(RAX) => String::from("rax"),
        Reg(RBX) => String::from("rbx"),
        Reg(RCX) => String::from("rcx"),
        Reg(RDX) => String::from("rdx"),
        Reg(RSP) => String::from("rsp"),
        Reg(RDI) => String::from("rdi"),
        Reg(RSI) => String::from("rsi"),
        Reg(RBP) => String::from("rbp"),
        Reg(R8) => String::from("r8"),
        Reg(R9) => String::from("r9"),
        ValTrue => String::from("3"),
        ValFalse => String::from("1"),
        Imm(n) => n.to_string(),
        RegOffset(RSP, n) => String::from(format!("[rsp - {}]", 8*n)),
        Label(s) => s,
        _ => panic!("Bad code generated!"),
    }
}

fn check_reg(val: Val) -> Val {
    matches!(val, Reg(_)).then(||val).expect("Codegen error - expected register!")
}

fn stringify(vec: Vec<Instr>) -> String {
    vec.into_iter().map(|i| match i {
        IMov(v1, v2) => format!("mov {}, {}", read_val(v1), read_val(v2)),
        IAdd(v1, v2) => format!("add {}, {}", read_val(v1), read_val(v2)),
        ISub(v1, v2) => format!("sub {}, {}", read_val(v1), read_val(v2)),
        IMul(v1, v2) => format!("imul {}, {}", read_val(check_reg(v1)), read_val(v2)),
        And(v1, v2) => format!("and {}, {}", read_val(v1), read_val(v2)),
        Xor(v1, v2) => format!("xor {}, {}", read_val(v1), read_val(v2)),
        Shl(v1, v2) => format!("shl {}, {}", read_val(v1), read_val(v2)),
        Sal(v1, v2) => format!("sal {}, {}", read_val(v1), read_val(v2)),
        Sar(v1, v2) => format!("sar {}, {}", read_val(v1), read_val(v2)),
        Cmp(v1, v2) => format!("cmp {}, {}", read_val(v1), read_val(v2)),
        Test(v1, v2) => format!("test {}, {}", read_val(v1), read_val(v2)),
        Cmovg(v1, v2) => format!("cmovg {}, {}", read_val(check_reg(v1)), read_val(v2)),
        Cmovge(v1, v2) => format!("cmovge {}, {}", read_val(check_reg(v1)), read_val(v2)),
        Cmovl(v1, v2) => format!("cmovl {}, {}", read_val(check_reg(v1)), read_val(v2)),
        Cmovle(v1, v2) => format!("cmovle {}, {}", read_val(check_reg(v1)), read_val(v2)),
        Cmovz(v1, v2) => format!("cmovz {}, {}", read_val(check_reg(v1)), read_val(v2)),
        Not(v) => format!("not {}", read_val(v)),
        Inc(v) => format!("inc {}", read_val(v)),
        ILabel(v) => format!("{}", read_val(v)),
        Jmp(v) => format!("jmp {}", read_val(v)),
        Je(v) => format!("je {}", read_val(v)),
        Jo(v) => format!("jo {}", read_val(v)),
        Jnz(v) => format!("jnz {}", read_val(v)),
        Call(v) => format!("call {}", read_val(v)),
        Ret => format!("ret\n"),
    }).collect::<Vec<String>>().iter().fold(String::new(), |accum, i| accum + "\n" + i)
}

pub fn compile_expr(e: &Expr) -> String {
    let mut instrs = Vec::new();
    let mut outer_namespace = Namespace{h: HashMap::new(), break_label: String::from(""), func: String::from("main")};
    let mut uid = 0;
    compile(e, &mut instrs, STACK_BASE, &mut uid, &mut outer_namespace);
    stringify(instrs)
}