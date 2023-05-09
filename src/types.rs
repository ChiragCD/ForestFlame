#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Val {
    Reg(Register),
    Imm(i64),
    RegOffset(Register, i64),
    ValTrue,
    ValFalse,
    Label(String),
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Register { RAX, RBX, RCX, RDX, RSP, RDI, RSI, RBP, R8, R9}

#[derive(Debug)]
pub enum Instr {
    IMov(Val, Val),
    IAdd(Val, Val),
    ISub(Val, Val),
    IMul(Val, Val),
    And(Val, Val),
    Xor(Val, Val),
    Not(Val),
    Shl(Val, Val),
    Sal(Val, Val),
    Sar(Val, Val),
    Cmp(Val, Val),
    Test(Val, Val),
    Cmovg(Val, Val),
    Cmovge(Val, Val),
    Cmovl(Val, Val),
    Cmovle(Val, Val),
    Cmovz(Val, Val),
    Inc(Val),
    ILabel(Val),
    Jmp(Val),
    Je(Val),
    Jo(Val),
    Jnz(Val),
    Call(Val),
    Ret,
}

#[derive(Debug)]
pub enum ZeroOp {
    Number(i64),
    OpTrue,
    OpFalse,
    Input,
    Identifier(String),
}

#[derive(Debug)]
pub enum UnitaryOp { Add1, Sub1, IsNum, IsBool, Loop, Break, }

#[derive(Debug)]
pub enum BinaryOp { Plus, Minus, Times, Less, Greater, LessEqual, GreaterEqual, Equal, }

#[derive(Debug)]
pub struct Binding (pub String, pub Box<Expr>);

#[derive(Debug)]
pub enum Expr {
    Let(Vec<Binding>, Box<Expr>),
    Set(Binding),
    EZeroOp(ZeroOp),
    EUnitaryOp(UnitaryOp, Box<Expr>),
    EBinaryOp(BinaryOp, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Block(Vec<Expr>),
    FuncDef(String, Vec<String>, Box<Expr>),
    FuncCall(String, Vec<Box<Expr>>),
    Program(Vec<Expr>, Box<Expr>),
}