use super::*;
use crate::color::RgbaColor;
use crate::geom::Unit;

/// An expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// The none literal: `none`.
    None,
    /// A identifier literal: `left`.
    Ident(Ident),
    /// A boolean literal: `true`, `false`.
    Bool(bool),
    /// An integer literal: `120`.
    Int(i64),
    /// A floating-point literal: `1.2`, `10e-4`.
    Float(f64),
    /// A length literal: `12pt`, `3cm`.
    Length(f64, Unit),
    /// A percent literal: `50%`.
    ///
    /// _Note_: `50%` is stored as `50.0` here, but as `0.5` in the
    /// corresponding [value](crate::geom::Relative).
    Percent(f64),
    /// A color literal: `#ffccee`.
    Color(RgbaColor),
    /// A string literal: `"hello!"`.
    Str(String),
    /// An invocation of a function: `[foo ...]`, `foo(...)`.
    Call(ExprCall),
    /// A unary operation: `-x`.
    Unary(ExprUnary),
    /// A binary operation: `a + b`, `a / b`.
    Binary(ExprBinary),
    /// An array expression: `(1, "hi", 12cm)`.
    Array(ExprArray),
    /// A dictionary expression: `(color: #f79143, pattern: dashed)`.
    Dict(ExprDict),
    /// A content expression: `{*Hello* there!}`.
    Content(ExprContent),
}

impl Pretty for Expr {
    fn pretty(&self, p: &mut Printer) {
        match self {
            Self::None => p.push_str("none"),
            Self::Ident(v) => p.push_str(&v),
            Self::Bool(v) => write!(p, "{}", v).unwrap(),
            Self::Int(v) => write!(p, "{}", v).unwrap(),
            Self::Float(v) => write!(p, "{}", v).unwrap(),
            Self::Length(v, u) => write!(p, "{}{}", v, u).unwrap(),
            Self::Percent(v) => write!(p, "{}%", v).unwrap(),
            Self::Color(v) => write!(p, "{}", v).unwrap(),
            Self::Str(s) => write!(p, "{:?}", &s).unwrap(),
            Self::Call(call) => call.pretty(p),
            Self::Unary(unary) => unary.pretty(p),
            Self::Binary(binary) => binary.pretty(p),
            Self::Array(array) => array.pretty(p),
            Self::Dict(dict) => dict.pretty(p),
            Self::Content(content) => pretty_content_expr(content, p),
        }
    }
}

/// Pretty print content in an expression context.
pub fn pretty_content_expr(tree: &Tree, p: &mut Printer) {
    if let [Spanned { v: Node::Expr(Expr::Call(call)), .. }] = tree.as_slice() {
        // Remove unncessary braces from content expression containing just a
        // single function call.
        //
        // Example: Transforms "{(call: {[f]})}" => "{(call: [f])}"
        pretty_bracket_call(call, p, false);
    } else {
        p.push_str("{");
        tree.pretty(p);
        p.push_str("}");
    }
}

/// An invocation of a function: `[foo ...]`, `foo(...)`.
#[derive(Debug, Clone, PartialEq)]
pub struct ExprCall {
    /// The name of the function.
    pub name: Spanned<Ident>,
    /// The arguments to the function.
    pub args: Spanned<ExprArgs>,
}

impl Pretty for ExprCall {
    fn pretty(&self, p: &mut Printer) {
        p.push_str(&self.name.v);
        p.push_str("(");
        self.args.v.pretty(p);
        p.push_str(")");
    }
}

/// Pretty print a bracketed function call, with body or chaining when possible.
pub fn pretty_bracket_call(call: &ExprCall, p: &mut Printer, chained: bool) {
    if chained {
        p.push_str(" | ");
    } else {
        p.push_str("[");
    }

    // Function name.
    p.push_str(&call.name.v);

    // Find out whether this can be written as body or chain.
    //
    // Example: Transforms "[v {Hi}]" => "[v][Hi]".
    if let [head @ .., Argument::Pos(Spanned { v: Expr::Content(content), .. })] =
        call.args.v.as_slice()
    {
        // Previous arguments.
        if !head.is_empty() {
            p.push_str(" ");
            p.join(head, ", ", |item, p| item.pretty(p));
        }

        // Find out whether this can written as a chain.
        //
        // Example: Transforms "[v][[f]]" => "[v | f]".
        if let [Spanned { v: Node::Expr(Expr::Call(call)), .. }] = content.as_slice() {
            return pretty_bracket_call(call, p, true);
        } else {
            p.push_str("][");
            content.pretty(p);
        }
    } else if !call.args.v.is_empty() {
        p.push_str(" ");
        call.args.v.pretty(p);
    }

    // Either end of header or end of body.
    p.push_str("]");
}

/// The arguments to a function: `12, draw: false`.
///
/// In case of a bracketed invocation with a body, the body is _not_
/// included in the span for the sake of clearer error messages.
pub type ExprArgs = Vec<Argument>;

impl Pretty for Vec<Argument> {
    fn pretty(&self, p: &mut Printer) {
        p.join(self, ", ", |item, p| item.pretty(p));
    }
}

/// An argument to a function call: `12` or `draw: false`.
#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    /// A positional arguments.
    Pos(Spanned<Expr>),
    /// A named argument.
    Named(Named),
}

impl Pretty for Argument {
    fn pretty(&self, p: &mut Printer) {
        match self {
            Self::Pos(expr) => expr.v.pretty(p),
            Self::Named(named) => named.pretty(p),
        }
    }
}

/// A pair of a name and an expression: `pattern: dashed`.
#[derive(Debug, Clone, PartialEq)]
pub struct Named {
    /// The name: `pattern`.
    pub name: Spanned<Ident>,
    /// The right-hand side of the pair: `dashed`.
    pub expr: Spanned<Expr>,
}

impl Pretty for Named {
    fn pretty(&self, p: &mut Printer) {
        p.push_str(&self.name.v);
        p.push_str(": ");
        self.expr.v.pretty(p);
    }
}

/// A unary operation: `-x`.
#[derive(Debug, Clone, PartialEq)]
pub struct ExprUnary {
    /// The operator: `-`.
    pub op: Spanned<UnOp>,
    /// The expression to operator on: `x`.
    pub expr: Box<Spanned<Expr>>,
}

impl Pretty for ExprUnary {
    fn pretty(&self, p: &mut Printer) {
        self.op.v.pretty(p);
        self.expr.v.pretty(p);
    }
}

/// A unary operator.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum UnOp {
    /// The negation operator: `-`.
    Neg,
}

impl Pretty for UnOp {
    fn pretty(&self, p: &mut Printer) {
        p.push_str(match self {
            Self::Neg => "-",
        });
    }
}

/// A binary operation: `a + b`, `a / b`.
#[derive(Debug, Clone, PartialEq)]
pub struct ExprBinary {
    /// The left-hand side of the operation: `a`.
    pub lhs: Box<Spanned<Expr>>,
    /// The operator: `+`.
    pub op: Spanned<BinOp>,
    /// The right-hand side of the operation: `b`.
    pub rhs: Box<Spanned<Expr>>,
}

impl Pretty for ExprBinary {
    fn pretty(&self, p: &mut Printer) {
        self.lhs.v.pretty(p);
        p.push_str(" ");
        self.op.v.pretty(p);
        p.push_str(" ");
        self.rhs.v.pretty(p);
    }
}

/// A binary operator.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BinOp {
    /// The addition operator: `+`.
    Add,
    /// The subtraction operator: `-`.
    Sub,
    /// The multiplication operator: `*`.
    Mul,
    /// The division operator: `/`.
    Div,
}

impl Pretty for BinOp {
    fn pretty(&self, p: &mut Printer) {
        p.push_str(match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
        });
    }
}

/// An array expression: `(1, "hi", 12cm)`.
pub type ExprArray = SpanVec<Expr>;

impl Pretty for ExprArray {
    fn pretty(&self, p: &mut Printer) {
        p.push_str("(");
        p.join(self, ", ", |item, p| item.v.pretty(p));
        if self.len() == 1 {
            p.push_str(",");
        }
        p.push_str(")");
    }
}

/// A dictionary expression: `(color: #f79143, pattern: dashed)`.
pub type ExprDict = Vec<Named>;

impl Pretty for ExprDict {
    fn pretty(&self, p: &mut Printer) {
        p.push_str("(");
        if self.is_empty() {
            p.push_str(":");
        } else {
            p.join(self, ", ", |named, p| named.pretty(p));
        }
        p.push_str(")");
    }
}

/// A content expression: `{*Hello* there!}`.
pub type ExprContent = Tree;

#[cfg(test)]
mod tests {
    use super::super::tests::test_pretty;

    #[test]
    fn test_pretty_print_chaining() {
        // All equivalent.
        test_pretty("[v [f]]", "[v | f]");
        test_pretty("[v {[f]}]", "[v | f]");
        test_pretty("[v][[f]]", "[v | f]");
        test_pretty("[v | f]", "[v | f]");
    }

    #[test]
    fn test_pretty_print_expressions() {
        // Unary and binary operations.
        test_pretty("{1 +}", "{1}");
        test_pretty("{1 + func(-2)}", "{1 + func(-2)}");

        // Array.
        test_pretty("(-5,)", "(-5,)");
        test_pretty("(1, 2, 3)", "(1, 2, 3)");

        // Dictionary.
        test_pretty("{(:)}", "{(:)}");
        test_pretty("{(percent: 5%)}", "{(percent: 5%)}");

        // Content expression without unncessary braces.
        test_pretty("[v [f], 1]", "[v [f], 1]");
        test_pretty("(func: {[f]})", "(func: [f])");
    }

    #[test]
    fn test_pretty_print_literals() {
        test_pretty("{none}", "{none}");
        test_pretty("{true}", "{true}");
        test_pretty("{25}", "{25}");
        test_pretty("{2.50}", "{2.5}");
        test_pretty("{1e2}", "{100}");
        test_pretty("{12pt}", "{12pt}");
        test_pretty("{50%}", "{50%}");
        test_pretty("{#fff}", "{#ffffff}");
        test_pretty(r#"{"hi\n"}"#, r#"{"hi\n"}"#);
    }
}
