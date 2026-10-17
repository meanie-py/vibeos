use crate::{Corpus, Rule, SourceSet};
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

pub struct StripMsvcCmake;
static INPUTS: &[SourceSet] =
    &[SourceSet::tracked("cmake", &["cmake"], &["CMakeLists.txt"]).with_inputs()];
static CMD_START: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^([ \t]*)([A-Za-z_][A-Za-z0-9_]*)[ \t]*\(").unwrap());
#[derive(Clone, Debug)]
enum Node {
    Atom(String),
    Or(Vec<Node>),
    And(Vec<Node>),
    Not(Box<Node>),
    Paren(Box<Node>),
    Const(bool),
}
fn tokens(s: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        if i == b.len() {
            break;
        }
        if b[i] == b'(' || b[i] == b')' {
            out.push((b[i] as char).to_string());
            i += 1;
            continue;
        }
        let start = i;
        if b[i] == b'"' {
            i += 1;
            while i < b.len() && b[i] != b'"' {
                if b[i] == b'\\' {
                    i += 1;
                }
                i += 1;
            }
            if i == b.len() {
                return Err("unterminated quoted token".into());
            }
            i += 1;
        } else {
            while i < b.len() && !b[i].is_ascii_whitespace() && b[i] != b'(' && b[i] != b')' {
                i += 1;
            }
        }
        out.push(s[start..i].to_string());
    }
    Ok(out)
}
const BINARY: &[&str] = &[
    "STREQUAL",
    "STRLESS",
    "STRGREATER",
    "STRLESS_EQUAL",
    "STRGREATER_EQUAL",
    "EQUAL",
    "LESS",
    "GREATER",
    "LESS_EQUAL",
    "GREATER_EQUAL",
    "MATCHES",
    "VERSION_LESS",
    "VERSION_GREATER",
    "VERSION_EQUAL",
    "VERSION_LESS_EQUAL",
    "VERSION_GREATER_EQUAL",
    "IN_LIST",
    "IS_NEWER_THAN",
    "PATH_EQUAL",
];
const UNARY: &[&str] = &[
    "EXISTS",
    "COMMAND",
    "DEFINED",
    "TARGET",
    "POLICY",
    "TEST",
    "IS_DIRECTORY",
    "IS_SYMLINK",
    "IS_ABSOLUTE",
    "IS_READABLE",
    "IS_WRITABLE",
    "IS_EXECUTABLE",
];
struct Parser {
    t: Vec<String>,
    i: usize,
}
impl Parser {
    fn peek(&self) -> Option<&str> {
        self.t.get(self.i).map(String::as_str)
    }
    fn take(&mut self) -> Result<String, String> {
        let x = self
            .t
            .get(self.i)
            .ok_or("unexpected end of condition")?
            .clone();
        self.i += 1;
        Ok(x)
    }
    fn parse(&mut self) -> Result<Node, String> {
        let n = self.or()?;
        if self.peek().is_some() {
            return Err(format!("trailing tokens: {:?}", &self.t[self.i..]));
        }
        Ok(n)
    }
    fn or(&mut self) -> Result<Node, String> {
        let mut v = vec![self.and()?];
        while self.peek() == Some("OR") {
            self.take()?;
            v.push(self.and()?);
        }
        Ok(if v.len() == 1 {
            v.remove(0)
        } else {
            Node::Or(v)
        })
    }
    fn and(&mut self) -> Result<Node, String> {
        let mut v = vec![self.not()?];
        while self.peek() == Some("AND") {
            self.take()?;
            v.push(self.not()?);
        }
        Ok(if v.len() == 1 {
            v.remove(0)
        } else {
            Node::And(v)
        })
    }
    fn not(&mut self) -> Result<Node, String> {
        if self.peek() == Some("NOT") {
            self.take()?;
            Ok(Node::Not(Box::new(self.not()?)))
        } else {
            self.primary()
        }
    }
    fn primary(&mut self) -> Result<Node, String> {
        let tok = self
            .peek()
            .ok_or("unexpected end of condition")?
            .to_string();
        if tok == "(" {
            self.take()?;
            let n = self.or()?;
            if self.take()? != ")" {
                return Err("unbalanced parenthesis".into());
            }
            return Ok(Node::Paren(Box::new(n)));
        }
        if UNARY.contains(&tok.as_str()) {
            self.take()?;
            return Ok(Node::Atom(format!("{} {}", tok, self.take()?)));
        }
        let left = self.take()?;
        if self.peek().is_some_and(|p| BINARY.contains(&p)) {
            let op = self.take()?;
            return Ok(Node::Atom(format!("{left} {op} {}", self.take()?)));
        }
        Ok(Node::Atom(left))
    }
}
fn atom_value(s: &str) -> Option<bool> {
    let p: Vec<_> = s.split_whitespace().collect();
    if p.len() == 1 && ["MSVC", "MSVC_IDE"].contains(&p[0]) {
        return Some(false);
    }
    if p.len() == 3
        && ["CMAKE_C_COMPILER_ID", "CMAKE_CXX_COMPILER_ID"].contains(&p[0])
        && p[1] == "STREQUAL"
        && p[2].trim_matches('"') == "MSVC"
    {
        return Some(false);
    }
    None
}
fn simplify(n: Node) -> (Node, bool) {
    match n {
        Node::Atom(s) => {
            if let Some(v) = atom_value(&s) {
                (Node::Const(v), true)
            } else {
                (Node::Atom(s), false)
            }
        }
        Node::Paren(x) => {
            let (i, c) = simplify(*x);
            if matches!(i, Node::Const(_)) {
                (i, true)
            } else {
                (Node::Paren(Box::new(i)), c)
            }
        }
        Node::Not(x) => {
            let (i, c) = simplify(*x);
            if let Node::Const(v) = i {
                (Node::Const(!v), true)
            } else {
                (Node::Not(Box::new(i)), c)
            }
        }
        Node::Or(xs) => simplify_logic(xs, true),
        Node::And(xs) => simplify_logic(xs, false),
        Node::Const(v) => (Node::Const(v), false),
    }
}
fn simplify_logic(xs: Vec<Node>, is_or: bool) -> (Node, bool) {
    let absorbing = is_or;
    let mut out = vec![];
    let mut changed = false;
    for x in xs {
        let (y, c) = simplify(x);
        changed |= c;
        if let Node::Const(v) = y {
            changed = true;
            if v == absorbing {
                return (Node::Const(absorbing), true);
            }
        } else {
            out.push(y)
        }
    }
    if out.is_empty() {
        return (Node::Const(!absorbing), true);
    }
    if out.len() == 1 {
        return (out.remove(0), true);
    }
    (if is_or { Node::Or(out) } else { Node::And(out) }, changed)
}
fn render(n: &Node) -> String {
    match n {
        Node::Atom(s) => s.clone(),
        Node::Paren(x) => format!("({})", render(x)),
        Node::Not(x) => format!("NOT {}", render(x)),
        Node::Const(v) => {
            if *v {
                "TRUE".into()
            } else {
                "FALSE".into()
            }
        }
        Node::Or(xs) => xs.iter().map(render).collect::<Vec<_>>().join(" OR "),
        Node::And(xs) => xs.iter().map(render).collect::<Vec<_>>().join(" AND "),
    }
}
fn evaluate(cond: &str) -> Result<(Option<bool>, Option<String>), String> {
    let mut p = Parser {
        t: tokens(cond)?,
        i: 0,
    };
    let (n, changed) = simplify(p.parse()?);
    if let Node::Const(v) = n {
        return Ok((Some(v), None));
    }
    let n = if let Node::Paren(x) = n { *x } else { n };
    Ok((None, if changed { Some(render(&n)) } else { None }))
}
fn contains_fact(n: &Node) -> bool {
    match n {
        Node::Atom(s) => atom_value(s).is_some(),
        Node::Not(x) | Node::Paren(x) => contains_fact(x),
        Node::Or(xs) | Node::And(xs) => xs.iter().any(contains_fact),
        Node::Const(_) => false,
    }
}
fn mentions_facts(s: &str) -> bool {
    tokens(s)
        .ok()
        .and_then(|t| Parser { t, i: 0 }.parse().ok())
        .is_some_and(|n| contains_fact(&n))
}
#[derive(Clone)]
struct Cmd {
    name: String,
    args: String,
    start: usize,
    end: usize,
    indent: String,
}
fn commands(text: &str) -> Result<Vec<Cmd>, String> {
    let mut out = vec![];
    let mut pos = 0;
    while let Some(m) = CMD_START.find_at(text, pos) {
        let caps = CMD_START.captures_at(text, m.start()).unwrap();
        let name = caps.get(2).unwrap().as_str().to_ascii_lowercase();
        let start = m.start();
        let open = m.end();
        let b = text.as_bytes();
        let mut i = open;
        let mut depth = 1;
        while i < b.len() {
            match b[i] {
                b'"' => {
                    i += 1;
                    while i < b.len() && b[i] != b'"' {
                        if b[i] == b'\\' {
                            i += 1;
                        }
                        i += 1
                    }
                    i += usize::from(i < b.len());
                }
                b'#' => {
                    while i < b.len() && b[i] != b'\n' {
                        i += 1
                    }
                }
                b'(' => {
                    depth += 1;
                    i += 1
                }
                b')' => {
                    depth -= 1;
                    i += 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => i += 1,
            }
        }
        if depth != 0 {
            return Err("unterminated command".into());
        }
        let close = i;
        let end = text[close..]
            .find('\n')
            .map_or(text.len(), |x| close + x + 1);
        out.push(Cmd {
            name,
            args: text[open..close - 1].to_string(),
            start,
            end,
            indent: caps.get(1).unwrap().as_str().to_string(),
        });
        pos = end;
    }
    Ok(out)
}
fn pending(text: &str) -> Result<usize, String> {
    Ok(commands(text)?
        .iter()
        .filter(|c| ["if", "elseif"].contains(&c.name.as_str()) && mentions_facts(&c.args))
        .count())
}

impl Rule for StripMsvcCmake {
    fn name(&self) -> &'static str {
        "strip_msvc_cmake"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let mut files: Vec<_> = c.sources("cmake").collect();
        files.sort_by_key(|f| f.path.clone());
        let mut left = 0;
        let mut out = vec![];
        for f in files {
            if f.path.starts_with("vendor")
                || (!f.path.file_name().is_some_and(|n| n == "CMakeLists.txt")
                    && f.path.extension().is_none_or(|e| e != "cmake"))
                || !f.text.contains("MSVC")
            {
                continue;
            }
            match pending(&f.text) {
                Ok(n) => {
                    if n > 0 {
                        left += n;
                        out.push(format!("{}: {} MSVC condition(s)", f.path.display(), n));
                    }
                }
                Err(_) => {
                    left += 1;
                }
            }
        }
        if left > 0 {
            out.push(format!("[strip_msvc_cmake] {left} MSVC condition(s) left -- MSVC is not a supported toolchain; resolve them with: python3 tools/strip_msvc_cmake.py --write"));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceFile;
    use std::path::PathBuf;
    #[test]
    fn condition_evaluation_matches_python_controls() {
        for (s,v,r) in [("MSVC",Some(false),None),("NOT MSVC",Some(true),None),("MSVC OR ARCH STREQUAL \"i386\"",None,Some("ARCH STREQUAL \"i386\"")),("NOT DEFINED MSVC_STATIC_RUNTIME AND MSVC",Some(false),None),("ARCH STREQUAL \"i386\"",None,None),("MSVC OR USE_NATIVE_SEH OR ((CMAKE_C_COMPILER_ID STREQUAL \"Clang\") AND (ARCH STREQUAL \"amd64\"))",None,Some("USE_NATIVE_SEH OR ((CMAKE_C_COMPILER_ID STREQUAL \"Clang\") AND (ARCH STREQUAL \"amd64\"))"))]{assert_eq!(evaluate(s).unwrap(),(v,r.map(str::to_string)),"{s}");}
    }
    #[test]
    fn scanner_control_bad_and_good() {
        let bad = "if(MSVC OR X)\n    a()\nendif()\n";
        let good = "if(CMAKE_C_COMPILER_ID STREQUAL \"Clang\")\n    a()\nendif()\n";
        assert_eq!(pending(bad).unwrap(), 1);
        assert_eq!(pending(good).unwrap(), 0);
    }
    #[test]
    fn nested_and_multiline_commands() {
        assert_eq!(
            pending("if (NOT MSVC AND\n    (ARCH STREQUAL \"i386\"))\n  a()\nendif()\n").unwrap(),
            1
        );
        assert_eq!(
            pending("IF ( X ) # MSVC in comment\n a()\nENDIF()\n").unwrap(),
            0
        );
    }
    #[test]
    fn rule_outputs_finding_lines_and_clean_empty() {
        let c = Corpus::new(
            PathBuf::from("."),
            vec![SourceFile::new(
                "CMakeLists.txt".into(),
                "if(MSVC OR X)\n a()\nendif()\n".into(),
            )],
        );
        assert_eq!(StripMsvcCmake.run(&c).unwrap(),vec!["CMakeLists.txt: 1 MSVC condition(s)".to_string(),"[strip_msvc_cmake] 1 MSVC condition(s) left -- MSVC is not a supported toolchain; resolve them with: python3 tools/strip_msvc_cmake.py --write".to_string()]);
    }
}
