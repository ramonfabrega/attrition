//! BHS — the original's scripting language, as an interpreter of our own.
//!
//! `docs/AI.md` §3: the skirmish AI's opening is `economic.bhs` or
//! `defensive.bhs`, shipped as data under `ai/scripts/` and run by the
//! engine's bytecode VM at production step 1. The scripts are data exactly
//! as the XML tables are, so they are read from the install and executed
//! here — by a tree-walker written from the language's *semantics*
//! (`~/ghidra-projects/reports/ai/bhs-language.md`, ratified in
//! `docs/AI.md` §11), not from the VM's design.
//!
//! The rules that differ from C, each pinned by a test below:
//!
//! - **truth is `> 0`** — a negative int is false, and the scripts rely on
//!   it (`if (find_inactive_build(...))` with `-1` for "none");
//! - **function arguments evaluate right to left**;
//! - `&&`/`||` short-circuit and always yield 0 or 1;
//! - a `static` initialiser runs **once, ever**, on the first execution by
//!   any caller — one slot per function shared by all eight leaders — and
//!   the slot is live for the *whole* of a call, statements above the
//!   declaration included, because the original keeps it on
//!   `Script::static_vars` rather than in the call's frame
//!   (`docs/AI.md` §17);
//! - a `trigger` block is inline code guarded by a bit that
//!   `enable_trigger` sets and that **firing clears**;
//! - `String` comparison is case-insensitive, and a mixed `String`/`int`
//!   operator casts the right operand to the left's type (else the left to
//!   the right's), an int becoming its decimal spelling;
//! - `/` and `%` truncate toward zero and wrap; a zero divisor is a
//!   runtime error, and **any runtime error aborts the whole run**;
//! - an assignment to an undeclared name declares it, function-scoped,
//!   with the value's type; identifiers are case-insensitive;
//! - falling off the end returns the type's default (0, `""`).
//!
//! Values are `int` and `String`. `float` exists in the language but no
//! shipped script uses it; a program that declares one fails to load
//! (§"What is not established"), which keeps this module under
//! `no_float.rs` until a software-float `float` is earned.
//!
//! Everything that persists between runs — statics, trigger bits — lives in
//! [`State`], which the simulation owns and digests. Timers are the host's
//! (`ScriptTimers` is engine state, keyed by string).

use std::collections::BTreeMap;

/// A script value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Int(i32),
    Str(String),
}

impl Value {
    /// `ScriptInt::is_false` is `value < 1`; a `String` is false when empty.
    pub fn truthy(&self) -> bool {
        match self {
            Value::Int(v) => *v > 0,
            Value::Str(s) => !s.is_empty(),
        }
    }

    pub const fn ty(&self) -> Ty {
        match self {
            Value::Int(_) => Ty::Int,
            Value::Str(_) => Ty::Str,
        }
    }

    /// `ScriptInt::get_string` / `ScriptString::get_int` — the two
    /// conversions the compiler inserts.
    pub fn cast(&self, to: Ty) -> Value {
        match (self, to) {
            (Value::Int(v), Ty::Str) => Value::Str(v.to_string()),
            (Value::Str(s), Ty::Int) => Value::Int(convert_int(s)),
            (v, _) => v.clone(),
        }
    }

    pub fn as_int(&self) -> i32 {
        match self {
            Value::Int(v) => *v,
            Value::Str(s) => convert_int(s),
        }
    }

    pub fn as_str(&self) -> String {
        match self {
            Value::Int(v) => v.to_string(),
            Value::Str(s) => s.clone(),
        }
    }
}

/// `String::convert_int`: a leading optional sign and decimal digits, 0
/// when there are none.
fn convert_int(s: &str) -> i32 {
    let t = s.trim_start();
    let (neg, digits) = match t.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let mut v: i32 = 0;
    for c in digits.chars() {
        match c.to_digit(10) {
            Some(d) => v = v.wrapping_mul(10).wrapping_add(d as i32),
            None => break,
        }
    }
    if neg { v.wrapping_neg() } else { v }
}

/// Case-insensitive string comparison, `_wcsicmp`'s order on ASCII.
fn str_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    a.chars()
        .map(|c| c.to_ascii_lowercase())
        .cmp(b.chars().map(|c| c.to_ascii_lowercase()))
}

/// The value types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ty {
    Int,
    Str,
    Void,
}

impl Ty {
    fn default_value(self) -> Value {
        match self {
            Ty::Int | Ty::Void => Value::Int(0),
            Ty::Str => Value::Str(String::new()),
        }
    }
}

/// What the interpreter asks of the engine: the host functions' signatures
/// at load time, and their execution at run time.
pub trait Host {
    /// The parameter types and return type of a host function taking
    /// `nargs` arguments, or `None` when no such function exists (a compile
    /// error in the script — `resolve_func_call` matches name and arity).
    fn signature(&self, name: &str, nargs: usize) -> Option<(Vec<Ty>, Ty)>;
    /// Run a host function. Arguments arrive already cast to the declared
    /// parameter types. An `Err` is a runtime error and aborts the run.
    fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, String>;
}

/// Why a program failed to load.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompileError {
    pub file: String,
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.file, self.line, self.message)
    }
}

/// Why a run failed — `RunTimeEnv::run_script`'s non-zero returns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunError {
    /// 3: no function of that name.
    NotFound,
    /// 4: the engine's arguments do not match the declaration.
    Params(String),
    /// 6: a runtime error at some statement — the run is abandoned.
    Runtime(String),
}

// ---------------------------------------------------------------- lexing

#[derive(Clone, Debug, PartialEq, Eq)]
enum Tok {
    Ident(String),
    Int(i32),
    Str(String),
    Punct(&'static str),
}

const PUNCTS: &[&str] = &[
    "<<=", ">>=", "==", "!=", "<=", ">=", "&&", "||", "++", "--", "+=", "-=", "*=", "/=", "%=",
    "&=", "|=", "^=", "<<", ">>", "**", "(", ")", "{", "}", "[", "]", ";", ",", "=", "<", ">", "+",
    "-", "*", "/", "%", "!", "&", "|", "^", "~", ":", "?", ".",
];

struct Lexed {
    toks: Vec<(Tok, usize)>,
}

fn lex(file: &str, src: &str) -> Result<Lexed, CompileError> {
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut line = 1;
    let mut toks = Vec::new();
    let err = |line: usize, m: &str| CompileError {
        file: file.to_string(),
        line,
        message: m.to_string(),
    };
    while i < chars.len() {
        let c = chars[i];
        if c == '\n' {
            line += 1;
            i += 1;
            continue;
        }
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '/' && chars.get(i + 1) == Some(&'*') {
            // Nested block comments, to `MAX_COMMENT_NEST`.
            let mut depth = 1;
            i += 2;
            while i < chars.len() && depth > 0 {
                if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                    depth += 1;
                    i += 2;
                } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    i += 2;
                } else {
                    if chars[i] == '\n' {
                        line += 1;
                    }
                    i += 1;
                }
            }
            continue;
        }
        if c == '"' {
            let mut s = String::new();
            i += 1;
            loop {
                match chars.get(i) {
                    None => return Err(err(line, "unterminated string")),
                    Some('"') => {
                        i += 1;
                        break;
                    }
                    Some('\\') => {
                        i += 1;
                        match chars.get(i) {
                            Some('n') => s.push('\n'),
                            Some('t') => s.push('\t'),
                            Some(&o) => s.push(o),
                            None => return Err(err(line, "unterminated string")),
                        }
                        i += 1;
                    }
                    Some(&o) => {
                        if o == '\n' {
                            line += 1;
                        }
                        s.push(o);
                        i += 1;
                    }
                }
            }
            toks.push((Tok::Str(s), line));
            continue;
        }
        if c.is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_alphanumeric() {
                i += 1;
            }
            let text: String = chars[start..i].iter().collect();
            if chars.get(i) == Some(&'.') && chars.get(i + 1).is_some_and(|d| d.is_ascii_digit()) {
                return Err(err(line, "decimal literals are not supported (no float)"));
            }
            let v = if let Some(h) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
                i64::from_str_radix(h, 16).map_err(|_| err(line, "bad number"))?
            } else {
                text.parse::<i64>().map_err(|_| err(line, "bad number"))?
            };
            toks.push((Tok::Int(v as i32), line));
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let text: String = chars[start..i]
                .iter()
                .map(|c| c.to_ascii_lowercase())
                .collect();
            toks.push((Tok::Ident(text), line));
            continue;
        }
        let mut matched = false;
        for p in PUNCTS {
            let n = p.len();
            if i + n <= chars.len() && chars[i..i + n].iter().copied().eq(p.chars()) {
                toks.push((Tok::Punct(p), line));
                i += n;
                matched = true;
                break;
            }
        }
        if !matched {
            return Err(err(line, &format!("unexpected character {c:?}")));
        }
    }
    Ok(Lexed { toks })
}

// ---------------------------------------------------------------- the AST

#[derive(Clone, Debug, PartialEq, Eq)]
enum Expr {
    Int(i32),
    Str(String),
    /// A frame slot.
    Var(usize),
    /// `x = e`, `x += e`, … on a frame slot.
    Assign(usize, AssignOp, Box<Expr>),
    /// `++x`, `x--`, …
    IncDec {
        slot: usize,
        inc: bool,
        pre: bool,
    },
    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    /// Short-circuit `&&` / `||`.
    Logical(bool, Box<Expr>, Box<Expr>),
    /// A script function by index, with its arguments; `ref` parameters are
    /// those whose argument is a plain variable and whose parameter is
    /// declared `ref`.
    CallScript(usize, Vec<Expr>),
    /// A host function by name, arguments cast to the declared types, and
    /// its return type.
    CallHost(String, Vec<Expr>, Vec<Ty>, Ty),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AssignOp {
    Set,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Shl,
    Shr,
    And,
    Or,
    Xor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnOp {
    Not,
    Neg,
    Tilde,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Shl,
    Shr,
    BitAnd,
    BitOr,
    BitXor,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Stmt {
    /// A declaration: the slot, the initialiser, and the static index when
    /// `static`.
    Decl {
        slot: usize,
        init: Option<Expr>,
        stat: Option<usize>,
        line: usize,
    },
    Expr(Expr, usize),
    If(Expr, Vec<Stmt>, Vec<Stmt>),
    While(Expr, Vec<Stmt>),
    DoWhile(Vec<Stmt>, Expr),
    For(Option<Expr>, Option<Expr>, Option<Expr>, Vec<Stmt>),
    Switch(Expr, Vec<(Option<Value>, Vec<Stmt>)>),
    Break,
    Continue,
    Return(Option<Expr>, usize),
    Block(Vec<Stmt>),
    /// `trigger name(cond) { body }`: the trigger bit index.
    Trigger(usize, Option<Expr>, Vec<Stmt>),
    /// `enable_trigger("name")`: the bit index.
    EnableTrigger(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Param {
    ty: Ty,
    by_ref: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Func {
    name: String,
    file: usize,
    ret: Ty,
    params: Vec<Param>,
    /// Slot types, parameters first.
    slots: Vec<Ty>,
    body: Vec<Stmt>,
    statics: usize,
    /// Every `static` declaration's `(index, frame slot)`, in body order —
    /// what [`Run::call`] seeds a call's frame from and what
    /// [`Run::sync_statics`] mirrors back. Precomputed because both run on
    /// the hot path.
    static_slots: Vec<(usize, usize)>,
    triggers: Vec<String>,
    defined: bool,
}

/// A loaded, resolved set of script files — the original's `ScriptFile`s
/// after `Compiler::compile` — immutable once built.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    files: Vec<String>,
    funcs: Vec<Func>,
}

/// The mutable script state the simulation owns: every function's statics
/// and trigger bits. `Default` is "nothing has run yet".
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    /// Per function: each static's value once initialised.
    statics: Vec<Vec<Option<Value>>>,
    /// Per function: each trigger's armed bit.
    triggers: Vec<Vec<bool>>,
}

impl State {
    pub fn new(p: &Program) -> State {
        State {
            statics: p.funcs.iter().map(|f| vec![None; f.statics]).collect(),
            triggers: p
                .funcs
                .iter()
                .map(|f| vec![false; f.triggers.len()])
                .collect(),
        }
    }

    /// Whether any static has been initialised — for tests and digests.
    pub fn any_static_set(&self) -> bool {
        self.statics.iter().flatten().any(Option::is_some)
    }

    /// A trigger's armed bit, by function and trigger name.
    pub fn trigger_armed(&self, p: &Program, func: &str, trigger: &str) -> Option<bool> {
        let fi = p.find(func)?;
        let ti = p.funcs[fi]
            .triggers
            .iter()
            .position(|t| t.eq_ignore_ascii_case(trigger))?;
        self.triggers.get(fi).and_then(|v| v.get(ti)).copied()
    }
}

// ---------------------------------------------------------------- parsing

/// A source file handed to the loader: its name (for `include`) and text.
pub struct Source<'a> {
    pub name: &'a str,
    pub text: &'a str,
}

struct Parser<'a> {
    file_name: String,
    toks: Vec<(Tok, usize)>,
    pos: usize,
    host: &'a dyn Host,
    /// Every file's `labels` constants, visible across includes.
    labels: &'a BTreeMap<String, Value>,
    /// Function name → index, across every file (case-folded).
    func_index: &'a BTreeMap<String, usize>,
    /// Each function's declared parameters and return type, for binding.
    func_sigs: &'a [(Vec<Param>, Ty)],
    // The function being parsed.
    /// Name, slot, and whether it was declared with a type (block-scoped)
    /// or implicitly by assignment (function-scoped).
    locals: Vec<(String, usize, bool)>,
    slots: Vec<Ty>,
    statics: usize,
    triggers: Vec<String>,
    /// Trigger names referenced by `enable_trigger` before definition.
    pending_triggers: Vec<(usize, String, usize)>,
}

fn is_type_name(s: &str) -> Option<Ty> {
    match s {
        "int" | "bool" => Some(Ty::Int),
        "string" => Some(Ty::Str),
        "void" => Some(Ty::Void),
        _ => None,
    }
}

impl<'a> Parser<'a> {
    fn err(&self, m: impl Into<String>) -> CompileError {
        CompileError {
            file: self.file_name.clone(),
            line: self.line(),
            message: m.into(),
        }
    }

    fn line(&self) -> usize {
        self.toks
            .get(self.pos)
            .or_else(|| self.toks.last())
            .map_or(0, |t| t.1)
    }

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos).map(|t| &t.0)
    }

    fn peek_at(&self, k: usize) -> Option<&Tok> {
        self.toks.get(self.pos + k).map(|t| &t.0)
    }

    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).map(|t| t.0.clone());
        self.pos += 1;
        t
    }

    fn is_punct(&self, p: &str) -> bool {
        matches!(self.peek(), Some(Tok::Punct(q)) if *q == p)
    }

    fn eat(&mut self, p: &str) -> bool {
        if self.is_punct(p) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, p: &str) -> Result<(), CompileError> {
        if self.eat(p) {
            Ok(())
        } else {
            Err(self.err(format!("expected `{p}`")))
        }
    }

    fn ident(&mut self) -> Result<String, CompileError> {
        match self.next() {
            Some(Tok::Ident(s)) => Ok(s),
            _ => Err(self.err("expected an identifier")),
        }
    }

    fn slot_of(&self, name: &str) -> Option<usize> {
        self.locals
            .iter()
            .rev()
            .find(|(n, _, _)| n == name)
            .map(|(_, s, _)| *s)
    }

    fn declare(&mut self, name: &str, ty: Ty, explicit: bool) -> usize {
        let s = self.slots.len();
        self.slots.push(ty);
        self.locals.push((name.to_string(), s, explicit));
        s
    }

    fn trigger_index(&mut self, name: &str) -> usize {
        if let Some(i) = self.triggers.iter().position(|t| t == name) {
            i
        } else {
            self.triggers.push(name.to_string());
            self.triggers.len() - 1
        }
    }

    // ---- statements

    fn block(&mut self) -> Result<Vec<Stmt>, CompileError> {
        self.expect("{")?;
        let mark = self.locals.len();
        let mut out = Vec::new();
        while !self.is_punct("}") {
            if self.peek().is_none() {
                return Err(self.err("unexpected end of file in a block"));
            }
            out.push(self.stmt()?);
        }
        self.expect("}")?;
        // Explicit declarations are block-scoped; implicit ones are
        // function-scoped, so only names declared with a type are dropped.
        let kept: Vec<_> = self
            .locals
            .drain(mark..)
            .filter(|(_, _, explicit)| !explicit)
            .collect();
        self.locals.extend(kept);
        Ok(out)
    }

    fn stmt(&mut self) -> Result<Stmt, CompileError> {
        let line = self.line();
        if self.eat(";") {
            return Ok(Stmt::Block(Vec::new()));
        }
        if self.is_punct("{") {
            return Ok(Stmt::Block(self.block()?));
        }
        let Some(Tok::Ident(word)) = self.peek().cloned() else {
            let e = self.expr()?;
            self.expect(";")?;
            return Ok(Stmt::Expr(e, line));
        };
        match word.as_str() {
            "if" => {
                self.pos += 1;
                self.expect("(")?;
                let c = self.expr()?;
                self.expect(")")?;
                let t = self.stmt()?;
                let e = if matches!(self.peek(), Some(Tok::Ident(w)) if w == "else") {
                    self.pos += 1;
                    vec![self.stmt()?]
                } else {
                    Vec::new()
                };
                Ok(Stmt::If(c, vec![t], e))
            }
            "while" => {
                self.pos += 1;
                self.expect("(")?;
                let c = self.expr()?;
                self.expect(")")?;
                let b = self.stmt()?;
                Ok(Stmt::While(c, vec![b]))
            }
            "do" => {
                self.pos += 1;
                let b = self.stmt()?;
                match self.next() {
                    Some(Tok::Ident(w)) if w == "while" => {}
                    _ => return Err(self.err("expected `while`")),
                }
                self.expect("(")?;
                let c = self.expr()?;
                self.expect(")")?;
                self.expect(";")?;
                Ok(Stmt::DoWhile(vec![b], c))
            }
            "for" => {
                self.pos += 1;
                self.expect("(")?;
                let init = if self.is_punct(";") {
                    None
                } else {
                    Some(self.expr()?)
                };
                self.expect(";")?;
                let cond = if self.is_punct(";") {
                    None
                } else {
                    Some(self.expr()?)
                };
                self.expect(";")?;
                let incr = if self.is_punct(")") {
                    None
                } else {
                    Some(self.expr()?)
                };
                self.expect(")")?;
                let b = self.stmt()?;
                Ok(Stmt::For(init, cond, incr, vec![b]))
            }
            "switch" => {
                self.pos += 1;
                self.expect("(")?;
                let subject = self.expr()?;
                self.expect(")")?;
                self.expect("{")?;
                let mut cases = Vec::new();
                while !self.is_punct("}") {
                    let label = match self.next() {
                        Some(Tok::Ident(w)) if w == "case" => {
                            let v = self.const_expr()?;
                            Some(v)
                        }
                        Some(Tok::Ident(w)) if w == "default" => None,
                        _ => return Err(self.err("expected `case` or `default`")),
                    };
                    self.expect(":")?;
                    let mut body = Vec::new();
                    while !self.is_punct("}")
                        && !matches!(self.peek(), Some(Tok::Ident(w)) if w == "case" || w == "default")
                    {
                        body.push(self.stmt()?);
                    }
                    cases.push((label, body));
                }
                self.expect("}")?;
                Ok(Stmt::Switch(subject, cases))
            }
            "break" => {
                self.pos += 1;
                self.expect(";")?;
                Ok(Stmt::Break)
            }
            "continue" => {
                self.pos += 1;
                self.expect(";")?;
                Ok(Stmt::Continue)
            }
            "return" => {
                self.pos += 1;
                if self.eat(";") {
                    return Ok(Stmt::Return(None, line));
                }
                let e = self.expr()?;
                self.expect(";")?;
                Ok(Stmt::Return(Some(e), line))
            }
            "trigger" => {
                self.pos += 1;
                let name = self.ident()?;
                let ti = self.trigger_index(&name);
                self.expect("(")?;
                let cond = if self.is_punct(")") {
                    None
                } else {
                    Some(self.expr()?)
                };
                self.expect(")")?;
                let body = self.block()?;
                Ok(Stmt::Trigger(ti, cond, body))
            }
            "enable_trigger" => {
                self.pos += 1;
                self.expect("(")?;
                let name = match self.next() {
                    Some(Tok::Str(s)) => s.to_ascii_lowercase(),
                    _ => return Err(self.err("enable_trigger takes a string")),
                };
                self.expect(")")?;
                self.expect(";")?;
                let ti = self.trigger_index(&name);
                self.pending_triggers.push((ti, name, line));
                Ok(Stmt::EnableTrigger(ti))
            }
            "static" => {
                self.pos += 1;
                let ty_name = self.ident()?;
                let ty = is_type_name(&ty_name).ok_or_else(|| self.err("expected a type"))?;
                self.decl_rest(ty, true, line)
            }
            "float" => Err(self.err("float is not supported")),
            _ => {
                if let Some(ty) = is_type_name(&word)
                    && matches!(self.peek_at(1), Some(Tok::Ident(_)))
                {
                    self.pos += 1;
                    return self.decl_rest(ty, false, line);
                }
                let e = self.expr()?;
                self.expect(";")?;
                Ok(Stmt::Expr(e, line))
            }
        }
    }

    fn decl_rest(&mut self, ty: Ty, is_static: bool, line: usize) -> Result<Stmt, CompileError> {
        if ty == Ty::Void {
            return Err(self.err("a variable cannot be void"));
        }
        let mut items = Vec::new();
        loop {
            let name = self.ident()?;
            let init = if self.eat("=") {
                Some(self.expr()?)
            } else {
                None
            };
            let slot = self.declare(&name, ty, true);
            let stat = if is_static {
                self.statics += 1;
                Some(self.statics - 1)
            } else {
                None
            };
            items.push(Stmt::Decl {
                slot,
                init,
                stat,
                line,
            });
            if !self.eat(",") {
                break;
            }
        }
        self.expect(";")?;
        Ok(if items.len() == 1 {
            items.pop().expect("one")
        } else {
            Stmt::Block(items)
        })
    }

    fn const_expr(&mut self) -> Result<Value, CompileError> {
        let neg = self.eat("-");
        let v = match self.next() {
            Some(Tok::Int(v)) => Value::Int(v),
            Some(Tok::Str(s)) => Value::Str(s),
            Some(Tok::Ident(name)) => self
                .labels
                .get(&name)
                .cloned()
                .ok_or_else(|| self.err(format!("`{name}` is not a constant")))?,
            _ => return Err(self.err("expected a constant")),
        };
        Ok(match (neg, v) {
            (true, Value::Int(i)) => Value::Int(i.wrapping_neg()),
            (true, _) => return Err(self.err("cannot negate a string")),
            (false, v) => v,
        })
    }

    // ---- expressions, by precedence

    fn expr(&mut self) -> Result<Expr, CompileError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, CompileError> {
        // `name op= expr`
        if let (Some(Tok::Ident(name)), Some(Tok::Punct(p))) =
            (self.peek().cloned(), self.peek_at(1).cloned())
        {
            let op = match p {
                "=" => Some(AssignOp::Set),
                "+=" => Some(AssignOp::Add),
                "-=" => Some(AssignOp::Sub),
                "*=" => Some(AssignOp::Mul),
                "/=" => Some(AssignOp::Div),
                "%=" => Some(AssignOp::Mod),
                "<<=" => Some(AssignOp::Shl),
                ">>=" => Some(AssignOp::Shr),
                "&=" => Some(AssignOp::And),
                "|=" => Some(AssignOp::Or),
                "^=" => Some(AssignOp::Xor),
                _ => None,
            };
            if let Some(op) = op {
                if is_type_name(&name).is_some() || self.labels.contains_key(&name) {
                    return Err(self.err(format!("cannot assign to `{name}`")));
                }
                self.pos += 2;
                let rhs = self.assignment()?;
                let slot = match self.slot_of(&name) {
                    Some(s) => s,
                    None => {
                        if op != AssignOp::Set {
                            return Err(self.err(format!("`{name}` is not declared")));
                        }
                        // An implicit variable takes the value's type.
                        let ty = self.type_of(&rhs);
                        self.declare(&name, ty, false)
                    }
                };
                return Ok(Expr::Assign(slot, op, Box::new(rhs)));
            }
        }
        self.logical_or()
    }

    fn type_of(&self, e: &Expr) -> Ty {
        match e {
            Expr::Int(_) | Expr::IncDec { .. } | Expr::Logical(..) => Ty::Int,
            Expr::Str(_) => Ty::Str,
            Expr::Var(s) | Expr::Assign(s, _, _) => self.slots[*s],
            Expr::Unary(op, inner) => match op {
                UnOp::Not | UnOp::Neg | UnOp::Tilde => {
                    if matches!(op, UnOp::Not) {
                        Ty::Int
                    } else {
                        self.type_of(inner)
                    }
                }
            },
            Expr::Binary(op, l, _) => match op {
                BinOp::Add => self.type_of(l),
                _ => Ty::Int,
            },
            Expr::CallScript(f, _) => self.func_sigs[*f].1,
            Expr::CallHost(_, _, _, ret) => *ret,
        }
    }

    fn logical_or(&mut self) -> Result<Expr, CompileError> {
        let mut l = self.logical_and()?;
        while self.eat("||") {
            let r = self.logical_and()?;
            l = Expr::Logical(true, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn logical_and(&mut self) -> Result<Expr, CompileError> {
        let mut l = self.bit_or()?;
        while self.eat("&&") {
            let r = self.bit_or()?;
            l = Expr::Logical(false, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn bit_or(&mut self) -> Result<Expr, CompileError> {
        let mut l = self.bit_xor()?;
        while self.is_punct("|") {
            self.pos += 1;
            let r = self.bit_xor()?;
            l = Expr::Binary(BinOp::BitOr, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn bit_xor(&mut self) -> Result<Expr, CompileError> {
        let mut l = self.bit_and()?;
        while self.is_punct("^") {
            self.pos += 1;
            let r = self.bit_and()?;
            l = Expr::Binary(BinOp::BitXor, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn bit_and(&mut self) -> Result<Expr, CompileError> {
        let mut l = self.equality()?;
        while self.is_punct("&") {
            self.pos += 1;
            let r = self.equality()?;
            l = Expr::Binary(BinOp::BitAnd, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn equality(&mut self) -> Result<Expr, CompileError> {
        let mut l = self.relational()?;
        loop {
            let op = if self.eat("==") {
                BinOp::Eq
            } else if self.eat("!=") {
                BinOp::Ne
            } else {
                break;
            };
            let r = self.relational()?;
            l = Expr::Binary(op, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn relational(&mut self) -> Result<Expr, CompileError> {
        let mut l = self.shift()?;
        loop {
            let op = if self.eat("<=") {
                BinOp::Le
            } else if self.eat(">=") {
                BinOp::Ge
            } else if self.eat("<") {
                BinOp::Lt
            } else if self.eat(">") {
                BinOp::Gt
            } else {
                break;
            };
            let r = self.shift()?;
            l = Expr::Binary(op, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn shift(&mut self) -> Result<Expr, CompileError> {
        let mut l = self.additive()?;
        loop {
            let op = if self.eat("<<") {
                BinOp::Shl
            } else if self.eat(">>") {
                BinOp::Shr
            } else {
                break;
            };
            let r = self.additive()?;
            l = Expr::Binary(op, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn additive(&mut self) -> Result<Expr, CompileError> {
        let mut l = self.multiplicative()?;
        loop {
            let op = if self.eat("+") {
                BinOp::Add
            } else if self.eat("-") {
                BinOp::Sub
            } else {
                break;
            };
            let r = self.multiplicative()?;
            l = Expr::Binary(op, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn multiplicative(&mut self) -> Result<Expr, CompileError> {
        let mut l = self.unary()?;
        loop {
            let op = if self.eat("*") {
                BinOp::Mul
            } else if self.eat("/") {
                BinOp::Div
            } else if self.eat("%") {
                BinOp::Mod
            } else if self.is_punct("**") {
                return Err(self.err("`**` is not supported (float)"));
            } else {
                break;
            };
            let r = self.unary()?;
            l = Expr::Binary(op, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn unary(&mut self) -> Result<Expr, CompileError> {
        if self.eat("!") {
            return Ok(Expr::Unary(UnOp::Not, Box::new(self.unary()?)));
        }
        if self.eat("-") {
            return Ok(Expr::Unary(UnOp::Neg, Box::new(self.unary()?)));
        }
        if self.eat("~") {
            return Ok(Expr::Unary(UnOp::Tilde, Box::new(self.unary()?)));
        }
        if self.eat("+") {
            return self.unary();
        }
        for (p, inc) in [("++", true), ("--", false)] {
            if self.eat(p) {
                let name = self.ident()?;
                let slot = self
                    .slot_of(&name)
                    .ok_or_else(|| self.err(format!("`{name}` is not declared")))?;
                return Ok(Expr::IncDec {
                    slot,
                    inc,
                    pre: true,
                });
            }
        }
        self.postfix()
    }

    fn postfix(&mut self) -> Result<Expr, CompileError> {
        let e = self.primary()?;
        if let Expr::Var(slot) = e {
            for (p, inc) in [("++", true), ("--", false)] {
                if self.eat(p) {
                    return Ok(Expr::IncDec {
                        slot,
                        inc,
                        pre: false,
                    });
                }
            }
        }
        Ok(e)
    }

    fn primary(&mut self) -> Result<Expr, CompileError> {
        match self.next() {
            Some(Tok::Int(v)) => Ok(Expr::Int(v)),
            Some(Tok::Str(s)) => Ok(Expr::Str(s)),
            Some(Tok::Punct("(")) => {
                let e = self.expr()?;
                self.expect(")")?;
                Ok(e)
            }
            Some(Tok::Ident(name)) => {
                if self.is_punct("(") {
                    return self.call(&name);
                }
                if let Some(v) = self.labels.get(&name) {
                    return Ok(match v {
                        Value::Int(i) => Expr::Int(*i),
                        Value::Str(s) => Expr::Str(s.clone()),
                    });
                }
                match self.slot_of(&name) {
                    Some(s) => Ok(Expr::Var(s)),
                    None => Err(self.err(format!("`{name}` is not declared"))),
                }
            }
            _ => Err(self.err("expected an expression")),
        }
    }

    fn call(&mut self, name: &str) -> Result<Expr, CompileError> {
        self.expect("(")?;
        let mut args = Vec::new();
        if !self.is_punct(")") {
            loop {
                args.push(self.expr()?);
                if !self.eat(",") {
                    break;
                }
            }
        }
        self.expect(")")?;
        if let Some(&fi) = self.func_index.get(name) {
            let (params, _) = &self.func_sigs[fi];
            if params.len() != args.len() {
                return Err(self.err(format!(
                    "`{name}` takes {} parameters, {} given",
                    params.len(),
                    args.len()
                )));
            }
            return Ok(Expr::CallScript(fi, args));
        }
        if let Some((params, ret)) = self.host.signature(name, args.len()) {
            if params.len() != args.len() {
                return Err(self.err(format!(
                    "host `{name}` takes {} parameters, {} given",
                    params.len(),
                    args.len()
                )));
            }
            return Ok(Expr::CallHost(name.to_string(), args, params, ret));
        }
        Err(self.err(format!("unknown function `{name}`")))
    }
}

/// `labels { A = 1, B, C, }` and `include "file"` at file scope, and the
/// function headers, on a first pass; bodies on a second, once every
/// function of every file is known.
struct Header {
    name: String,
    ret: Ty,
    params: Vec<(String, Param)>,
    body_start: Option<usize>,
    line: usize,
}

fn file_scan(
    file: usize,
    file_name: &str,
    toks: &[(Tok, usize)],
    labels: &mut BTreeMap<String, Value>,
    includes: &mut Vec<String>,
) -> Result<Vec<Header>, CompileError> {
    let err = |line: usize, m: String| CompileError {
        file: file_name.to_string(),
        line,
        message: m,
    };
    let mut headers = Vec::new();
    let mut i = 0;
    let line_at = |i: usize| toks.get(i).or(toks.last()).map_or(0, |t| t.1);
    while i < toks.len() {
        match &toks[i].0 {
            Tok::Ident(w) if w == "labels" => {
                i += 1;
                if !matches!(toks.get(i), Some((Tok::Punct("{"), _))) {
                    return Err(err(line_at(i), "expected `{` after labels".into()));
                }
                i += 1;
                let mut prev: Option<i32> = None;
                loop {
                    match toks.get(i) {
                        Some((Tok::Punct("}"), _)) => {
                            i += 1;
                            break;
                        }
                        Some((Tok::Ident(name), _)) => {
                            i += 1;
                            let v = if matches!(toks.get(i), Some((Tok::Punct("="), _))) {
                                i += 1;
                                let neg = matches!(toks.get(i), Some((Tok::Punct("-"), _)));
                                if neg {
                                    i += 1;
                                }
                                match toks.get(i) {
                                    Some((Tok::Int(v), _)) => {
                                        i += 1;
                                        if neg { v.wrapping_neg() } else { *v }
                                    }
                                    _ => return Err(err(line_at(i), "expected a number".into())),
                                }
                            } else {
                                prev.map_or(1, |p| p.wrapping_add(1))
                            };
                            prev = Some(v);
                            labels.insert(name.clone(), Value::Int(v));
                            if matches!(toks.get(i), Some((Tok::Punct(","), _))) {
                                i += 1;
                            }
                        }
                        _ => return Err(err(line_at(i), "bad labels block".into())),
                    }
                }
            }
            Tok::Ident(w) if w == "include" => {
                i += 1;
                match toks.get(i) {
                    Some((Tok::Str(s), _)) => {
                        includes.push(s.clone());
                        i += 1;
                    }
                    _ => return Err(err(line_at(i), "include takes a string".into())),
                }
                if matches!(toks.get(i), Some((Tok::Punct(";"), _))) {
                    i += 1;
                }
            }
            Tok::Ident(w) if is_type_name(w).is_some() || w == "float" => {
                let line = toks[i].1;
                if w == "float" {
                    return Err(err(line, "float is not supported".into()));
                }
                let ret = is_type_name(w).expect("a type");
                i += 1;
                // `[tag] name (`
                let mut name = match toks.get(i) {
                    Some((Tok::Ident(n), _)) => n.clone(),
                    _ => return Err(err(line, "expected a function name".into())),
                };
                i += 1;
                if let Some((Tok::Ident(n), _)) = toks.get(i) {
                    // The first identifier was a tag (`ai`).
                    name = n.clone();
                    i += 1;
                }
                if !matches!(toks.get(i), Some((Tok::Punct("("), _))) {
                    return Err(err(line, "expected `(`".into()));
                }
                i += 1;
                let mut params = Vec::new();
                while !matches!(toks.get(i), Some((Tok::Punct(")"), _))) {
                    let mut by_ref = false;
                    if matches!(toks.get(i), Some((Tok::Ident(w), _)) if w == "ref") {
                        by_ref = true;
                        i += 1;
                    }
                    let ty = match toks.get(i) {
                        Some((Tok::Ident(t), _)) => match is_type_name(t) {
                            Some(Ty::Void) | None => {
                                return Err(err(line, format!("bad parameter type `{t}`")));
                            }
                            Some(t) => t,
                        },
                        _ => return Err(err(line, "expected a parameter type".into())),
                    };
                    i += 1;
                    let pname = match toks.get(i) {
                        Some((Tok::Ident(n), _)) => n.clone(),
                        _ => return Err(err(line, "expected a parameter name".into())),
                    };
                    i += 1;
                    params.push((pname, Param { ty, by_ref }));
                    if matches!(toks.get(i), Some((Tok::Punct(","), _))) {
                        i += 1;
                    }
                }
                i += 1;
                let body_start = if matches!(toks.get(i), Some((Tok::Punct(";"), _))) {
                    i += 1;
                    None
                } else if matches!(toks.get(i), Some((Tok::Punct("{"), _))) {
                    let start = i;
                    // Skip the balanced body.
                    let mut depth = 0;
                    loop {
                        match toks.get(i) {
                            Some((Tok::Punct("{"), _)) => depth += 1,
                            Some((Tok::Punct("}"), _)) => {
                                depth -= 1;
                                if depth == 0 {
                                    i += 1;
                                    break;
                                }
                            }
                            None => return Err(err(line, "unterminated function body".into())),
                            _ => {}
                        }
                        i += 1;
                    }
                    Some(start)
                } else {
                    return Err(err(line, "expected `;` or `{`".into()));
                };
                headers.push(Header {
                    name,
                    ret,
                    params,
                    body_start,
                    line,
                });
            }
            _ => {
                return Err(err(
                    toks[i].1,
                    format!("unexpected token at file scope: {:?}", toks[i].0),
                ));
            }
        }
    }
    let _ = file;
    Ok(headers)
}

impl Program {
    /// Load a program from its root files, pulling `include`s from `sources`
    /// by name (case-insensitive, base name only). The host's signatures
    /// bind host calls at load time, as `Compiler::compile` does.
    pub fn load(
        roots: &[&str],
        sources: &[Source],
        host: &dyn Host,
    ) -> Result<Program, CompileError> {
        let find = |name: &str| -> Option<&Source> {
            let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
            sources.iter().find(|s| {
                let b = s.name.rsplit(['/', '\\']).next().unwrap_or(s.name);
                b.eq_ignore_ascii_case(base)
            })
        };
        // Load order: each root, then its includes (each file once).
        let mut order: Vec<&Source> = Vec::new();
        let mut queue: Vec<String> = roots.iter().map(|r| r.to_string()).collect();
        let mut lexed: Vec<Lexed> = Vec::new();
        let mut labels = BTreeMap::new();
        let mut headers: Vec<Vec<Header>> = Vec::new();
        while let Some(name) = queue.first().cloned() {
            queue.remove(0);
            let Some(src) = find(&name) else {
                return Err(CompileError {
                    file: name.clone(),
                    line: 0,
                    message: "no such script file".into(),
                });
            };
            if order.iter().any(|s| std::ptr::eq(*s, src)) {
                continue;
            }
            let fi = order.len();
            order.push(src);
            let lx = lex(src.name, src.text)?;
            let mut includes = Vec::new();
            let hs = file_scan(fi, src.name, &lx.toks, &mut labels, &mut includes)?;
            lexed.push(lx);
            headers.push(hs);
            queue.extend(includes);
        }
        // Function table: a declaration and its definition are one entry.
        let mut funcs: Vec<Func> = Vec::new();
        let mut func_index: BTreeMap<String, usize> = BTreeMap::new();
        let mut bodies: Vec<(usize, usize, usize)> = Vec::new(); // (func, file, tok start)
        for (fi, hs) in headers.iter().enumerate() {
            for h in hs {
                let idx = match func_index.get(&h.name) {
                    Some(&i) => {
                        let f = &funcs[i];
                        if f.ret != h.ret
                            || f.params.len() != h.params.len()
                            || f.params
                                .iter()
                                .zip(h.params.iter())
                                .any(|(a, (_, b))| a != b)
                        {
                            return Err(CompileError {
                                file: order[fi].name.to_string(),
                                line: h.line,
                                message: format!("redeclaration of `{}` differs", h.name),
                            });
                        }
                        i
                    }
                    None => {
                        funcs.push(Func {
                            name: h.name.clone(),
                            file: fi,
                            ret: h.ret,
                            params: h.params.iter().map(|(_, p)| p.clone()).collect(),
                            slots: Vec::new(),
                            body: Vec::new(),
                            statics: 0,
                            static_slots: Vec::new(),
                            triggers: Vec::new(),
                            defined: false,
                        });
                        func_index.insert(h.name.clone(), funcs.len() - 1);
                        funcs.len() - 1
                    }
                };
                if let Some(start) = h.body_start {
                    if funcs[idx].defined {
                        return Err(CompileError {
                            file: order[fi].name.to_string(),
                            line: h.line,
                            message: format!("`{}` already implemented", h.name),
                        });
                    }
                    funcs[idx].defined = true;
                    funcs[idx].file = fi;
                    bodies.push((idx, fi, start));
                }
            }
        }
        let func_sigs: Vec<(Vec<Param>, Ty)> =
            funcs.iter().map(|f| (f.params.clone(), f.ret)).collect();
        // Bodies.
        for (idx, fi, start) in bodies {
            let header = headers[fi]
                .iter()
                .find(|h| h.name == funcs[idx].name && h.body_start == Some(start))
                .expect("its header");
            let mut p = Parser {
                file_name: order[fi].name.to_string(),
                toks: lexed[fi].toks.clone(),
                pos: start,
                host,
                labels: &labels,
                func_index: &func_index,
                func_sigs: &func_sigs,
                locals: Vec::new(),
                slots: Vec::new(),
                statics: 0,
                triggers: Vec::new(),
                pending_triggers: Vec::new(),
            };
            for (name, param) in &header.params {
                p.declare(name, param.ty, true);
            }
            let body = p.block()?;
            for (ti, name, line) in &p.pending_triggers {
                let _ = ti;
                // A trigger enabled but never defined in this function is
                // `resolve_trigger_toggle`'s "Unknown trigger" error.
                if !body_defines_trigger(&body, p.triggers.iter().position(|t| t == name)) {
                    return Err(CompileError {
                        file: order[fi].name.to_string(),
                        line: *line,
                        message: format!("unknown trigger `{name}`"),
                    });
                }
            }
            let f = &mut funcs[idx];
            f.slots = p.slots;
            f.static_slots = Vec::new();
            collect_statics(&body, &mut f.static_slots);
            f.body = body;
            f.statics = p.statics;
            f.triggers = p.triggers;
        }
        Ok(Program {
            files: order.iter().map(|s| s.name.to_string()).collect(),
            funcs,
        })
    }

    /// A function's index by name, case-insensitively, newest file first
    /// (`ScriptFile::find_script`).
    pub fn find(&self, name: &str) -> Option<usize> {
        let mut best: Option<usize> = None;
        for (i, f) in self.funcs.iter().enumerate() {
            if f.defined && f.name.eq_ignore_ascii_case(name) {
                match best {
                    Some(b) if self.funcs[b].file >= f.file => {}
                    _ => best = Some(i),
                }
            }
        }
        best
    }

    pub fn files(&self) -> &[String] {
        &self.files
    }
}

/// Every `static` declaration in a body, as `(static index, frame slot)`.
///
/// A `static`'s frame slot is an ordinary local that the run seeds from the
/// persistent store on entry and mirrors back on every write, so the whole
/// body — including the statements *above* the declaration — sees the value
/// the last call left. The walk recurses through every construct a
/// declaration can sit in.
fn collect_statics(body: &[Stmt], out: &mut Vec<(usize, usize)>) {
    for s in body {
        match s {
            Stmt::Decl {
                slot,
                stat: Some(si),
                ..
            } => out.push((*si, *slot)),
            Stmt::If(_, a, b) => {
                collect_statics(a, out);
                collect_statics(b, out);
            }
            Stmt::While(_, b)
            | Stmt::DoWhile(b, _)
            | Stmt::For(_, _, _, b)
            | Stmt::Block(b)
            | Stmt::Trigger(_, _, b) => collect_statics(b, out),
            Stmt::Switch(_, cases) => {
                for (_, b) in cases {
                    collect_statics(b, out);
                }
            }
            _ => {}
        }
    }
}

fn body_defines_trigger(body: &[Stmt], ti: Option<usize>) -> bool {
    let Some(ti) = ti else { return false };
    fn walk(s: &Stmt, ti: usize) -> bool {
        match s {
            Stmt::Trigger(t, _, b) => *t == ti || b.iter().any(|s| walk(s, ti)),
            Stmt::If(_, a, b) => a.iter().chain(b).any(|s| walk(s, ti)),
            Stmt::While(_, b) | Stmt::DoWhile(b, _) | Stmt::For(_, _, _, b) | Stmt::Block(b) => {
                b.iter().any(|s| walk(s, ti))
            }
            Stmt::Switch(_, cases) => cases.iter().any(|(_, b)| b.iter().any(|s| walk(s, ti))),
            _ => false,
        }
    }
    body.iter().any(|s| walk(s, ti))
}

// ---------------------------------------------------------------- running

/// `BYTECODE_MAX`: the original's watchdog, counted here in evaluation
/// steps rather than opcodes — a bound of the same order.
const STEP_MAX: u64 = 0x100_0000;

enum Flow {
    Next,
    Break,
    Continue,
    Return(Value),
}

/// One run of a script function — `RunTimeEnv::run_script`. Arguments are
/// passed as values; a `ref` parameter's final value is written back into
/// `args` on success, which is what the engine reads after the run.
pub fn run(
    program: &Program,
    state: &mut State,
    host: &mut dyn Host,
    name: &str,
    args: &mut [Value],
) -> Result<Value, RunError> {
    let fi = program.find(name).ok_or(RunError::NotFound)?;
    let f = &program.funcs[fi];
    if f.params.len() != args.len() {
        return Err(RunError::Params(format!(
            "`{name}` expects {} params, got {}",
            f.params.len(),
            args.len()
        )));
    }
    for (i, (p, a)) in f.params.iter().zip(args.iter()).enumerate() {
        if p.ty != a.ty() {
            return Err(RunError::Params(format!(
                "parameter {i} of `{name}` expects {:?}",
                p.ty
            )));
        }
    }
    let mut vm = Vm {
        program,
        state,
        host,
        steps: 0,
        depth: 0,
    };
    let (ret, back) = vm.call(fi, args.to_vec())?;
    for (i, p) in f.params.iter().enumerate() {
        if p.by_ref {
            args[i] = back[i].clone();
        }
    }
    Ok(ret)
}

struct Vm<'a> {
    program: &'a Program,
    state: &'a mut State,
    host: &'a mut dyn Host,
    steps: u64,
    depth: usize,
}

impl Vm<'_> {
    fn tick(&mut self) -> Result<(), RunError> {
        self.steps += 1;
        if self.steps > STEP_MAX {
            return Err(RunError::Runtime("script watchdog: too many steps".into()));
        }
        Ok(())
    }

    /// Call a script function; returns its value and its final parameter
    /// slots (for `ref` write-back).
    fn call(&mut self, fi: usize, args: Vec<Value>) -> Result<(Value, Vec<Value>), RunError> {
        if self.depth >= 1024 {
            return Err(RunError::Runtime("script stack too deep".into()));
        }
        self.depth += 1;
        let f = &self.program.funcs[fi];
        let mut vars: Vec<Value> = f.slots.iter().map(|t| t.default_value()).collect();
        for (i, a) in args.into_iter().enumerate() {
            vars[i] = a;
        }
        // Seed the `static` slots from the store *before* the body runs.
        // `VirtualMachine::get_value/set_value` read and write a static
        // straight on `Script::static_vars`, never through the call's frame,
        // so a static is visible from the first instruction of every call;
        // a frame that starts it at zero loses what the last call wrote
        // (`docs/AI.md` §17).
        self.seed_statics(fi, &mut vars);
        let f = &self.program.funcs[fi];
        let body = &f.body;
        let flow = self.block(fi, body, &mut vars)?;
        self.depth -= 1;
        let ret = match flow {
            Flow::Return(v) => v.cast(f.ret),
            _ => f.ret.default_value(),
        };
        let back = vars[..f.params.len()].to_vec();
        Ok((ret, back))
    }

    fn block(
        &mut self,
        fi: usize,
        stmts: &[Stmt],
        vars: &mut Vec<Value>,
    ) -> Result<Flow, RunError> {
        for s in stmts {
            match self.stmt(fi, s, vars)? {
                Flow::Next => {}
                other => return Ok(other),
            }
        }
        Ok(Flow::Next)
    }

    fn stmt(&mut self, fi: usize, s: &Stmt, vars: &mut Vec<Value>) -> Result<Flow, RunError> {
        self.tick()?;
        match s {
            Stmt::Decl {
                slot,
                init,
                stat,
                line: _,
            } => {
                let ty = self.program.funcs[fi].slots[*slot];
                match stat {
                    Some(si) => {
                        // Once, ever: the slot keeps its value across calls
                        // and callers; the initialiser runs on the first
                        // execution only.
                        if self.state.statics[fi][*si].is_none() {
                            let v = match init {
                                Some(e) => self.eval(fi, e, vars)?.cast(ty),
                                None => ty.default_value(),
                            };
                            self.state.statics[fi][*si] = Some(v);
                        }
                        vars[*slot] = self.state.statics[fi][*si].clone().expect("set");
                    }
                    None => {
                        let v = match init {
                            Some(e) => self.eval(fi, e, vars)?.cast(ty),
                            None => ty.default_value(),
                        };
                        vars[*slot] = v;
                    }
                }
                Ok(Flow::Next)
            }
            Stmt::Expr(e, _) => {
                self.eval(fi, e, vars)?;
                self.sync_statics(fi, vars);
                Ok(Flow::Next)
            }
            Stmt::If(c, t, e) => {
                if self.eval(fi, c, vars)?.truthy() {
                    self.block(fi, t, vars)
                } else {
                    self.block(fi, e, vars)
                }
            }
            Stmt::While(c, b) => {
                while self.eval(fi, c, vars)?.truthy() {
                    match self.block(fi, b, vars)? {
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Next | Flow::Continue => {}
                    }
                }
                Ok(Flow::Next)
            }
            Stmt::DoWhile(b, c) => {
                loop {
                    match self.block(fi, b, vars)? {
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Next | Flow::Continue => {}
                    }
                    if !self.eval(fi, c, vars)?.truthy() {
                        break;
                    }
                }
                Ok(Flow::Next)
            }
            Stmt::For(init, cond, incr, b) => {
                if let Some(e) = init {
                    self.eval(fi, e, vars)?;
                }
                loop {
                    if let Some(c) = cond
                        && !self.eval(fi, c, vars)?.truthy()
                    {
                        break;
                    }
                    match self.block(fi, b, vars)? {
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Next | Flow::Continue => {}
                    }
                    if let Some(e) = incr {
                        self.eval(fi, e, vars)?;
                    }
                }
                Ok(Flow::Next)
            }
            Stmt::Switch(subject, cases) => {
                let v = self.eval(fi, subject, vars)?;
                // The matching case, else `default`, else nothing; then
                // fall through as C does.
                let mut start = cases.iter().position(|(label, _)| match label {
                    Some(l) => values_equal(&v, l),
                    None => false,
                });
                if start.is_none() {
                    start = cases.iter().position(|(label, _)| label.is_none());
                }
                if let Some(s) = start {
                    for (_, body) in &cases[s..] {
                        match self.block(fi, body, vars)? {
                            Flow::Break => return Ok(Flow::Next),
                            Flow::Continue => return Ok(Flow::Continue),
                            Flow::Return(v) => return Ok(Flow::Return(v)),
                            Flow::Next => {}
                        }
                    }
                }
                Ok(Flow::Next)
            }
            Stmt::Break => Ok(Flow::Break),
            Stmt::Continue => Ok(Flow::Continue),
            Stmt::Return(e, _) => {
                let v = match e {
                    Some(e) => self.eval(fi, e, vars)?,
                    None => Value::Int(0),
                };
                self.sync_statics(fi, vars);
                Ok(Flow::Return(v))
            }
            Stmt::Block(b) => self.block(fi, b, vars),
            Stmt::Trigger(ti, cond, body) => {
                if !self.state.triggers[fi][*ti] {
                    return Ok(Flow::Next);
                }
                if let Some(c) = cond
                    && !self.eval(fi, c, vars)?.truthy()
                {
                    return Ok(Flow::Next);
                }
                // Firing disarms.
                self.state.triggers[fi][*ti] = false;
                self.block(fi, body, vars)
            }
            Stmt::EnableTrigger(ti) => {
                self.state.triggers[fi][*ti] = true;
                Ok(Flow::Next)
            }
        }
    }

    /// The frame's `static` slots, from the store: `get_value`'s
    /// `0x40000000` operand, which does not go through the frame at all.
    fn seed_statics(&self, fi: usize, vars: &mut [Value]) {
        for &(si, slot) in &self.program.funcs[fi].static_slots {
            if let Some(v) = &self.state.statics[fi][si] {
                vars[slot] = v.clone();
            }
        }
    }

    /// The mirror back: a write to a `static`'s frame slot reaches the
    /// store, so the next call — and this call's next read — sees it. Only
    /// a slot the store already holds is mirrored, which is what keeps a
    /// declaration's "once, ever" initialiser from being pre-empted by the
    /// frame's default.
    fn sync_statics(&mut self, fi: usize, vars: &[Value]) {
        for i in 0..self.program.funcs[fi].static_slots.len() {
            let (si, slot) = self.program.funcs[fi].static_slots[i];
            if self.state.statics[fi][si].is_some() {
                self.state.statics[fi][si] = Some(vars[slot].clone());
            }
        }
    }

    fn eval(&mut self, fi: usize, e: &Expr, vars: &mut Vec<Value>) -> Result<Value, RunError> {
        self.tick()?;
        Ok(match e {
            Expr::Int(v) => Value::Int(*v),
            Expr::Str(s) => Value::Str(s.clone()),
            Expr::Var(s) => vars[*s].clone(),
            Expr::Assign(slot, op, rhs) => {
                let r = self.eval(fi, rhs, vars)?;
                let ty = self.program.funcs[fi].slots[*slot];
                let cur = vars[*slot].clone();
                let v = match op {
                    AssignOp::Set => r.cast(ty),
                    AssignOp::Add => binary(BinOp::Add, cur, r)?,
                    AssignOp::Sub => binary(BinOp::Sub, cur, r)?,
                    AssignOp::Mul => binary(BinOp::Mul, cur, r)?,
                    AssignOp::Div => binary(BinOp::Div, cur, r)?,
                    AssignOp::Mod => binary(BinOp::Mod, cur, r)?,
                    AssignOp::Shl => binary(BinOp::Shl, cur, r)?,
                    AssignOp::Shr => binary(BinOp::Shr, cur, r)?,
                    AssignOp::And => binary(BinOp::BitAnd, cur, r)?,
                    AssignOp::Or => binary(BinOp::BitOr, cur, r)?,
                    AssignOp::Xor => binary(BinOp::BitXor, cur, r)?,
                };
                vars[*slot] = v.clone();
                v
            }
            Expr::IncDec { slot, inc, pre } => {
                let old = vars[*slot].as_int();
                let new = if *inc {
                    old.wrapping_add(1)
                } else {
                    old.wrapping_sub(1)
                };
                vars[*slot] = Value::Int(new);
                Value::Int(if *pre { new } else { old })
            }
            Expr::Unary(op, inner) => {
                let v = self.eval(fi, inner, vars)?;
                match op {
                    UnOp::Not => Value::Int(i32::from(!v.truthy())),
                    UnOp::Neg => match v {
                        Value::Int(i) => Value::Int(i.wrapping_neg()),
                        Value::Str(_) => {
                            return Err(RunError::Runtime("Can't do - on String".into()));
                        }
                    },
                    UnOp::Tilde => match v {
                        Value::Int(i) => Value::Int(!i),
                        Value::Str(_) => {
                            return Err(RunError::Runtime("Can't do ~ on String".into()));
                        }
                    },
                }
            }
            Expr::Binary(op, l, r) => {
                let lv = self.eval(fi, l, vars)?;
                let rv = self.eval(fi, r, vars)?;
                binary(*op, lv, rv)?
            }
            Expr::Logical(is_or, l, r) => {
                let lv = self.eval(fi, l, vars)?.truthy();
                if *is_or {
                    if lv {
                        Value::Int(1)
                    } else {
                        Value::Int(i32::from(self.eval(fi, r, vars)?.truthy()))
                    }
                } else if !lv {
                    Value::Int(0)
                } else {
                    Value::Int(i32::from(self.eval(fi, r, vars)?.truthy()))
                }
            }
            Expr::CallScript(callee, args) => {
                // Right to left.
                let mut vals = vec![Value::Int(0); args.len()];
                for (i, a) in args.iter().enumerate().rev() {
                    vals[i] = self.eval(fi, a, vars)?;
                }
                let params = &self.program.funcs[*callee].params;
                for (v, p) in vals.iter_mut().zip(params) {
                    *v = v.cast(p.ty);
                }
                self.sync_statics(fi, vars);
                let (ret, back) = self.call(*callee, vals)?;
                // A recursive call shares the store, so read it back.
                self.seed_statics(fi, vars);
                // `ref` write-back into the caller's variables.
                for (i, (p, a)) in params.iter().zip(args.iter()).enumerate() {
                    if p.by_ref
                        && let Expr::Var(s) = a
                    {
                        vars[*s] = back[i].clone();
                    }
                }
                ret
            }
            Expr::CallHost(name, args, tys, _) => {
                let mut vals = vec![Value::Int(0); args.len()];
                for (i, a) in args.iter().enumerate().rev() {
                    vals[i] = self.eval(fi, a, vars)?;
                }
                for (v, t) in vals.iter_mut().zip(tys) {
                    *v = v.cast(*t);
                }
                self.host.call(name, &vals).map_err(RunError::Runtime)?
            }
        })
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x.len() == y.len() && str_cmp(x, y).is_eq(),
        (Value::Int(x), Value::Str(y)) | (Value::Str(y), Value::Int(x)) => {
            let s = x.to_string();
            s.len() == y.len() && str_cmp(&s, y).is_eq()
        }
    }
}

/// A binary operator on two values — `ScriptInt::do_operator` /
/// `ScriptString::do_operator`, after the compiler's cast of a mixed pair.
fn binary(op: BinOp, l: Value, r: Value) -> Result<Value, RunError> {
    // A mixed pair: the right operand takes the left's type, else the left
    // the right's (both directions convert between int and String).
    let (l, r) = match (l, r) {
        (Value::Int(a), Value::Str(b)) => (Value::Str(a.to_string()), Value::Str(b)),
        (Value::Str(a), Value::Int(b)) => (Value::Str(a), Value::Str(b.to_string())),
        pair => pair,
    };
    Ok(match (l, r) {
        (Value::Int(a), Value::Int(b)) => Value::Int(match op {
            BinOp::Add => a.wrapping_add(b),
            BinOp::Sub => a.wrapping_sub(b),
            BinOp::Mul => a.wrapping_mul(b),
            BinOp::Div => {
                if b == 0 {
                    return Err(RunError::Runtime("Can't Divide 0".into()));
                }
                a.wrapping_div(b)
            }
            BinOp::Mod => {
                if b == 0 {
                    return Err(RunError::Runtime("Can't Divide 0".into()));
                }
                a.wrapping_rem(b)
            }
            BinOp::Eq => i32::from(a == b),
            BinOp::Ne => i32::from(a != b),
            BinOp::Lt => i32::from(a < b),
            BinOp::Gt => i32::from(a > b),
            BinOp::Le => i32::from(a <= b),
            BinOp::Ge => i32::from(a >= b),
            BinOp::Shl => a.wrapping_shl(b as u32 & 0x1f),
            BinOp::Shr => a.wrapping_shr(b as u32 & 0x1f),
            BinOp::BitAnd => a & b,
            BinOp::BitOr => a | b,
            BinOp::BitXor => a ^ b,
        }),
        (Value::Str(a), Value::Str(b)) => match op {
            BinOp::Add => Value::Str(a + &b),
            BinOp::Eq => Value::Int(i32::from(a.len() == b.len() && str_cmp(&a, &b).is_eq())),
            BinOp::Ne => Value::Int(i32::from(!(a.len() == b.len() && str_cmp(&a, &b).is_eq()))),
            BinOp::Lt => Value::Int(i32::from(str_cmp(&a, &b).is_lt())),
            BinOp::Gt => Value::Int(i32::from(str_cmp(&a, &b).is_gt())),
            BinOp::Le => Value::Int(i32::from(
                str_cmp(&a, &b).is_lt() || (a.len() == b.len() && str_cmp(&a, &b).is_eq()),
            )),
            BinOp::Ge => Value::Int(i32::from(
                str_cmp(&a, &b).is_gt() || (a.len() == b.len() && str_cmp(&a, &b).is_eq()),
            )),
            _ => {
                return Err(RunError::Runtime(format!(
                    "Can't do {op:?} on String and String"
                )));
            }
        },
        _ => unreachable!("mixed pairs are cast above"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A host with a few functions and a log of the calls it saw.
    #[derive(Default)]
    struct TestHost {
        log: Vec<(String, Vec<Value>)>,
        counter: i32,
    }

    impl Host for TestHost {
        fn signature(&self, name: &str, _nargs: usize) -> Option<(Vec<Ty>, Ty)> {
            Some(match name {
                "mark" => (vec![Ty::Int], Ty::Int),
                "next" => (vec![], Ty::Int),
                "name_of" => (vec![Ty::Int], Ty::Str),
                "set_timer" => (vec![Ty::Str, Ty::Int], Ty::Int),
                "minus_one" => (vec![], Ty::Int),
                _ => return None,
            })
        }
        fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, String> {
            self.log.push((name.to_string(), args.to_vec()));
            Ok(match name {
                "mark" => args[0].clone(),
                "next" => {
                    self.counter += 1;
                    Value::Int(self.counter)
                }
                "name_of" => Value::Str(if args[0].as_int() == 1 {
                    "Capital".into()
                } else {
                    String::new()
                }),
                "set_timer" => Value::Int(1),
                "minus_one" => Value::Int(-1),
                _ => return Err(format!("no host function {name}")),
            })
        }
    }

    fn load(src: &str) -> (Program, State, TestHost) {
        let host = TestHost::default();
        let p = Program::load(
            &["main.bhs"],
            &[Source {
                name: "main.bhs",
                text: src,
            }],
            &host,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        let s = State::new(&p);
        (p, s, host)
    }

    fn run_int(p: &Program, s: &mut State, h: &mut TestHost, f: &str, args: &mut [Value]) -> i32 {
        run(p, s, h, f, args).unwrap().as_int()
    }

    #[test]
    fn truth_is_greater_than_zero() {
        let (p, mut s, mut h) = load(
            "int ai t(int x) { if (x) return 10; return 20; }
             int ai n(int x) { return !x; }",
        );
        assert_eq!(run_int(&p, &mut s, &mut h, "t", &mut [Value::Int(1)]), 10);
        assert_eq!(run_int(&p, &mut s, &mut h, "t", &mut [Value::Int(0)]), 20);
        assert_eq!(
            run_int(&p, &mut s, &mut h, "t", &mut [Value::Int(-1)]),
            20,
            "-1 is false"
        );
        assert_eq!(run_int(&p, &mut s, &mut h, "n", &mut [Value::Int(-5)]), 1);
        assert_eq!(run_int(&p, &mut s, &mut h, "n", &mut [Value::Int(3)]), 0);
    }

    #[test]
    fn arguments_evaluate_right_to_left() {
        let (p, mut s, mut h) = load(
            "int ai two(int a, int b) { return a * 10 + b; }
             int ai go() { return two(next(), next()); }",
        );
        // `next()` yields 1 then 2; the second argument is evaluated first.
        assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut []), 2 * 10 + 1);
    }

    #[test]
    fn short_circuit_skips_the_host_call_and_yields_a_bit() {
        let (p, mut s, mut h) = load(
            "int ai go(int x) { return (x > 0) && mark(7); }
             int ai orr(int x) { return (x > 0) || mark(9); }",
        );
        assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(0)]), 0);
        assert!(h.log.is_empty(), "mark not called");
        assert_eq!(
            run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(1)]),
            1,
            "0/1, not 7"
        );
        assert_eq!(h.log.len(), 1);
        assert_eq!(run_int(&p, &mut s, &mut h, "orr", &mut [Value::Int(1)]), 1);
        assert_eq!(h.log.len(), 1);
    }

    #[test]
    fn a_static_initialises_once_ever_and_persists() {
        let (p, mut s, mut h) = load(
            "int ai go(int who) {
               static int seed = next();
               static int hits = 0;
               hits++;
               return seed * 100 + hits;
             }",
        );
        assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(1)]), 101);
        assert_eq!(
            run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(2)]),
            102,
            "shared by callers"
        );
        assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(1)]), 103);
        assert_eq!(h.counter, 1, "the initialiser ran once");
    }

    /// A statement above the declarations does not wipe the statics.
    ///
    /// `VirtualMachine::get_value`/`set_value` read and write a static on
    /// `Script::static_vars` rather than through the call's frame, so the
    /// value the last call left is live from the first instruction. This
    /// simulation gives a static a frame slot and mirrors it back, and the
    /// mirror used to run over slots the call had not yet declared — so any
    /// script with an expression statement above its `static` block lost
    /// every one of them on its *second* call, and only on the second.
    /// `economic.bhs` has three such statements, and its `needed_citizens`
    /// went back to zero on every call after the first: the AI stopped
    /// training citizens for the rest of the game (`docs/AI.md` §17).
    #[test]
    fn a_statement_above_the_declarations_does_not_wipe_the_statics() {
        let (p, mut s, mut h) = load(
            "int ai go(int who) {
               mark(who);
               static int hits = 0;
               hits++;
               return hits;
             }",
        );
        assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(1)]), 1);
        assert_eq!(
            run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(1)]),
            2,
            "the second call is the one that used to lose it"
        );
        assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(1)]), 3);
    }

    #[test]
    fn ref_parameters_write_back_and_labels_count_from_one() {
        let (p, mut s, mut h) = load(
            "labels { BLOCK_ON_THIS = 1, DONT_BLOCK_ON_THIS, SCRIPT_DONE, }
             int ai economic(int who, ref int step, int bvr, int loops) {
               step += 2;
               if (step > 4) return SCRIPT_DONE;
               return BLOCK_ON_THIS;
             }",
        );
        let mut args = [Value::Int(1), Value::Int(1), Value::Int(2), Value::Int(5)];
        assert_eq!(run_int(&p, &mut s, &mut h, "economic", &mut args), 1);
        assert_eq!(args[1], Value::Int(3), "the ref came back");
        assert_eq!(run_int(&p, &mut s, &mut h, "economic", &mut args), 3);
        assert_eq!(args[1], Value::Int(5));
    }

    #[test]
    fn triggers_are_one_shot_inline_and_rearm() {
        let (p, mut s, mut h) = load(
            "int ai place(int who) {
               new_city = 0;
               enable_trigger(\"city_build\");
               trigger city_build() {
                 if (mark(who) > 1) { enable_trigger(\"health_check\"); }
                 else { enable_trigger(\"city_build\"); }
               }
               trigger health_check() {
                 return 1;
               }
               return -1;
             }",
        );
        // who = 1: city_build fires, re-arms itself (deferred), returns -1.
        assert_eq!(
            run_int(&p, &mut s, &mut h, "place", &mut [Value::Int(1)]),
            -1
        );
        assert_eq!(s.trigger_armed(&p, "place", "city_build"), Some(true));
        // who = 2: city_build fires and arms health_check, which runs in
        // the same call and returns 1.
        assert_eq!(
            run_int(&p, &mut s, &mut h, "place", &mut [Value::Int(2)]),
            1
        );
        assert_eq!(
            s.trigger_armed(&p, "place", "health_check"),
            Some(false),
            "fired, so cleared"
        );
    }

    #[test]
    fn switch_falls_through_and_break_leaves_only_the_switch() {
        let (p, mut s, mut h) = load(
            "int ai go(int n) {
               int acc = 0;
               for (i = 0; i < 3; i++) {
                 switch (n) {
                 case 1: acc += 1;
                 case 2: acc += 10; break;
                 default: acc += 100; break;
                 }
               }
               return acc;
             }",
        );
        assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(1)]), 33);
        assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(2)]), 30);
        assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(9)]), 300);
    }

    #[test]
    fn strings_compare_case_insensitively_and_mix_with_ints() {
        let (p, mut s, mut h) = load(
            "int ai go() {
               String n = name_of(1);
               int a = (n == \"CAPITAL\");
               int b = (n > -1);
               String none = name_of(2);
               int c = (none > -1);
               int d = !none;
               return a * 1000 + b * 100 + c * 10 + d;
             }",
        );
        assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut []), 1101);
    }

    #[test]
    fn host_arguments_are_cast_to_the_declared_types() {
        let (p, mut s, mut h) = load("int ai go(int who) { return set_timer(who, 300); }");
        run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(3)]);
        assert_eq!(h.log[0].1, vec![Value::Str("3".into()), Value::Int(300)]);
    }

    #[test]
    fn division_truncates_and_a_zero_divisor_aborts() {
        let (p, mut s, mut h) = load("int ai go(int a, int b) { return a / b * 100 + a % b; }");
        assert_eq!(
            run_int(
                &p,
                &mut s,
                &mut h,
                "go",
                &mut [Value::Int(-7), Value::Int(2)]
            ),
            -300 - 1
        );
        assert!(matches!(
            run(
                &p,
                &mut s,
                &mut h,
                "go",
                &mut [Value::Int(1), Value::Int(0)]
            ),
            Err(RunError::Runtime(_))
        ));
    }

    #[test]
    fn implicit_variables_are_function_scoped_and_typed_by_first_assignment() {
        let (p, mut s, mut h) = load(
            "int ai go() {
               if (1) { who_nation = name_of(1); }
               if (who_nation == \"Capital\") return 1;
               return 0;
             }",
        );
        assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut []), 1);
    }

    #[test]
    fn falling_off_the_end_returns_the_default_and_not_found_is_an_error() {
        let (p, mut s, mut h) = load("int ai go() { int x = 3; } String ai nm() { }");
        assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut []), 0);
        assert_eq!(
            run(&p, &mut s, &mut h, "nm", &mut []).unwrap(),
            Value::Str(String::new())
        );
        assert_eq!(
            run(&p, &mut s, &mut h, "nope", &mut []),
            Err(RunError::NotFound)
        );
        assert!(matches!(
            run(&p, &mut s, &mut h, "go", &mut [Value::Int(1)]),
            Err(RunError::Params(_))
        ));
    }

    #[test]
    fn includes_share_labels_and_functions_and_a_library_static_is_shared() {
        let host = TestHost::default();
        let p = Program::load(
            &["a.bhs", "b.bhs"],
            &[
                Source {
                    name: "a.bhs",
                    text: "include \"lib.bhs\"\n int ai a(int who) { return helper(who) + DONE; }",
                },
                Source {
                    name: "b.bhs",
                    text: "include \"lib.bhs\"\n int ai b(int who) { return helper(who); }",
                },
                Source {
                    name: "lib.bhs",
                    text: "labels { DONE = 3, }\n int ai helper(int who);\n int ai helper(int who) { static int calls = 0; calls++; return calls; }",
                },
            ],
            &host,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        let mut s = State::new(&p);
        let mut h = host;
        assert_eq!(
            run_int(&p, &mut s, &mut h, "a", &mut [Value::Int(1)]),
            1 + 3
        );
        assert_eq!(
            run_int(&p, &mut s, &mut h, "b", &mut [Value::Int(1)]),
            2,
            "one library, one static"
        );
        assert_eq!(p.files().len(), 3);
    }

    /// A host that accepts any function at any arity, answering `1` — enough
    /// to load the shipped scripts and walk their first call.
    struct AnyHost;

    impl Host for AnyHost {
        fn signature(&self, name: &str, nargs: usize) -> Option<(Vec<Ty>, Ty)> {
            let ret = match name {
                "find_nation" | "find_city_with_num" | "get_mapstyle" => Ty::Str,
                _ => Ty::Int,
            };
            let mut params = vec![Ty::Int; nargs];
            // The string-taking parameters the shipped scripts use.
            for (i, p) in params.iter_mut().enumerate() {
                let is_str = match name {
                    "num_type"
                    | "num_type_with_queued"
                    | "have_tech"
                    | "can_pay_cost"
                    | "research_tech_with_cost"
                    | "researching_tech"
                    | "find_build"
                    | "find_inactive_build"
                    | "find_unit"
                    | "find_num_idle_unit" => i == 1,
                    "at_least_type" | "train_unit_with_cost" | "train_unit_at_with_cost" => i == 2,
                    "place_building_with_cost" | "place_building_upgrade_with_cost" => i >= 1,
                    "place_orphan_building_with_cost" => i == 1,
                    "find_build_at_city" | "num_city_buildings" => i == 1 || i == 2,
                    "num_type_queued" => i == 2,
                    "find_city_id" => i == 0,
                    "was_city_attacked" | "was_city_raided" => i == 1,
                    "set_timer" | "stop_timer" | "timer_expired" => i == 0,
                    _ => false,
                };
                if is_str {
                    *p = Ty::Str;
                }
            }
            Some((params, ret))
        }
        fn call(&mut self, name: &str, _args: &[Value]) -> Result<Value, String> {
            Ok(match name {
                "find_nation" | "find_city_with_num" | "get_mapstyle" => Value::Str("x".into()),
                _ => Value::Int(1),
            })
        }
    }

    /// The three shipped scripts load and run their first call — gated on
    /// the install, as every install-backed test is.
    #[test]
    fn the_shipped_scripts_load_and_run_a_first_call() {
        let Some(install) = crate::testenv::install_root() else {
            eprintln!("skipping: no install (set RON_INSTALL)");
            return;
        };
        let dir = std::path::Path::new(&install).join("ai").join("scripts");
        let read =
            |n: &str| std::fs::read_to_string(dir.join(n)).unwrap_or_else(|e| panic!("{n}: {e}"));
        let (e, d, l) = (
            read("economic.bhs"),
            read("defensive.bhs"),
            read("aibestbuildlibrary.bhs"),
        );
        let sources = [
            Source {
                name: "economic.bhs",
                text: &e,
            },
            Source {
                name: "defensive.bhs",
                text: &d,
            },
            Source {
                name: "aibestbuildlibrary.bhs",
                text: &l,
            },
        ];
        let p = Program::load(&["economic.bhs", "defensive.bhs"], &sources, &AnyHost)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(p.files().len(), 3, "the library is loaded once");
        let mut s = State::new(&p);
        let mut h = AnyHost;
        for name in ["economic", "defensive"] {
            let mut args = [Value::Int(2), Value::Int(1), Value::Int(2), Value::Int(5)];
            let r = run(&p, &mut s, &mut h, name, &mut args)
                .unwrap_or_else(|e| panic!("{name}: {e:?}"));
            // Every host answer is 1: "attacked" → a barracks placed → SCRIPT_DONE.
            assert_eq!(r, Value::Int(3), "{name}");
        }
    }

    #[test]
    fn a_float_declaration_fails_to_load() {
        let host = TestHost::default();
        let r = Program::load(
            &["m.bhs"],
            &[Source {
                name: "m.bhs",
                text: "int ai go() { float f = 1; return 0; }",
            }],
            &host,
        );
        assert!(r.is_err());
    }

    #[test]
    fn the_shipped_shapes_parse() {
        // The constructs the three shipped scripts use, in one function.
        let (p, mut s, mut h) = load(
            "//Mark Sobota\n
             labels {\n BLOCK_ON_THIS = 1,\n DONT_BLOCK_ON_THIS,\n SCRIPT_DONE,\n}
             int ai assign_idle (int who);
             int ai assign_idle (int who) { return -1;; }
             int ai economic (int who, ref int step, int boom_vs_rush, int num_loops)
             {
               if ((mark(who) < 1)&&(mark(who) < 1)) return SCRIPT_DONE;
               my_capital = name_of(who);
               int return_value = BLOCK_ON_THIS;
               static int prev_step0 = 0;
               static int needed_techs = mark(who);
               int old_step;
               switch (who-1) {
               case 0:
                 old_step = prev_step0;
                 break;
               default:
                 old_step = 0;
                 break;
               }
               for (j = mark(who); j < 3; j++) {
                 if (mark(j) < 1) break;
               }
               while (mark(who) < 0) { break; }
               if (step == 1) {
                 if (mark(who) > 4 || mark(who) > 1) { return SCRIPT_DONE; }
                 size = mark(2);
                 if (size == 0) step = 1;
                 else if (size >= 2) step = 6;
                 else return SCRIPT_DONE;
               }
               for (i = 0; i < num_loops; i++) {
                 if (old_step != step) old_step = step;
                 switch (step) {
                 case 6:
                   step++;
                   return_value = BLOCK_ON_THIS;
                   break;
                 case 7:
                   step+=2;
                   return BLOCK_ON_THIS;
                   break;
                 default:
                   return_value = SCRIPT_DONE;
                   break;
                 }
               }
               switch (who-1) {
               case 0:
                 prev_step0 = old_step;
                 break;
               }
               return return_value;
             }",
        );
        let mut args = [Value::Int(1), Value::Int(1), Value::Int(2), Value::Int(5)];
        assert_eq!(run_int(&p, &mut s, &mut h, "economic", &mut args), 1);
        assert_eq!(args[1], Value::Int(9), "6 → 7 → +2");
        assert!(s.any_static_set());
    }
}
