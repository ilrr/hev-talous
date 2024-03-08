use std::{
    borrow::BorrowMut,
    fmt,
    iter::{self, from_fn},
};

pub enum Sexpr {
    Atom(A),
    List(Vec<Sexpr>),
}

pub enum A {
    String(String),
    Number(i32),
    Symbol(String),
}

impl fmt::Display for Sexpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Sexpr::Atom(a) => write!(f, "{}", a.to_string()),
            Sexpr::List(l) => write!(
                f,
                "({})",
                l.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
        }
    }
}

impl Sexpr {
    pub fn push(&mut self, v: Sexpr) {
        match self {
            Self::List(l) => l.push(v),
            Self::Atom(_) => panic!("Can't push to atom!"),
        }
    }
    pub fn append(&mut self, v: Sexpr) {
        match self {
            Self::List(l) => match v {
                Sexpr::List(mut m) => l.append(m.borrow_mut()),
                _ => panic!("Can't append an atom"),
            },
            Self::Atom(_) => panic!("Can't append to an atom!"),
        }
    }

    pub fn push_last(&mut self, v: Sexpr) {
        match self {
            Self::List(l) => match l.last_mut() {
                Some(Self::List(m)) => m.push(v),
                _ => panic!(),
            },
            _ => panic!(),
        }
    }
    pub fn set_last(&mut self, v: Sexpr) {
        match self {
            Self::List(l) => {
                l.pop();
                l.push(v)
            }
            _ => panic!(),
        }
    }
    pub fn get(&self, i: usize) -> &Sexpr {
        match self {
            Self::List(l) => l.get(i).unwrap(),
            Self::Atom(_) => panic!("Cant index atom!"),
        }
    }
    pub fn get_mut(&mut self, i: usize) -> &Sexpr {
        match self {
            Self::List(l) => l.get_mut(i).unwrap(),
            Self::Atom(_) => panic!("Cant index atom!"),
        }
    }
    pub fn mutate_5_5(&mut self, v: Sexpr) {
        if let Sexpr::List(l1) = self {
            if let Some(s1) = l1.get_mut(5) {
                if let Sexpr::List(l2) = s1 {
                    if let Some(s2) = l2.get_mut(5) {
                        *s2 = v;
                    }
                }
            }
        }
    }
}

impl fmt::Display for A {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            A::String(s) => write!(f, "\"{}\"", s),
            A::Number(n) => write!(f, "{}", n),
            A::Symbol(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Debug for Sexpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

trait Stack<T> {
    fn new() -> Self;
    fn push(&mut self, v: T) -> ();
    fn pop(&mut self) -> T;
    fn push_top(&mut self, v: T) -> ();
    fn push_empty(&mut self) -> ();
}

impl Stack<Sexpr> for Vec<Sexpr> {
    fn new() -> Self {
        Vec::new()
    }
    fn push(&mut self, v: Sexpr) -> () {
        self.push(v);
    }
    fn pop(&mut self) -> Sexpr {
        self.pop().expect("Empty stack")
    }
    fn push_top(&mut self, v: Sexpr) -> () {
        self.last_mut().expect("Error").push(v);
    }
    fn push_empty(&mut self) -> () {
        self.push(Sexpr::List(Vec::new()));
    }
}

pub fn parse(s: String) -> Sexpr {
    let mut stack: Vec<Sexpr> = Vec::new();
    let mut iter = s.chars().peekable();
    while let Some(ch) = iter.next() {
        match ch {
            '(' => stack.push_empty(),
            ')' => {
                let top = stack.pop().expect("Unmatched parentheses");
                // println!("1- {top}");
                if stack.is_empty() {
                    return top;
                }
                stack.push_top(top);
            }
            '"' => {
                let s: String = iter::once(ch)
                    .chain(from_fn(|| iter.by_ref().next_if(|c| *c != '"')))
                    .collect();
                iter.next();
                // println!("{s}");
                stack.push_top(Sexpr::Atom(A::String(s[1..].to_string())));
            }
            '0'..='9' | '-' => {
                let n: i32 = iter::once(ch)
                    .chain(from_fn(|| iter.by_ref().next_if(|c| c.is_ascii_digit())))
                    .collect::<String>()
                    .parse()
                    .unwrap();
                stack.push_top(Sexpr::Atom(A::Number(n)))
            }
            c if c.is_whitespace() => {}

            _ => {
                let s: String = iter::once(ch)
                    .chain(from_fn(|| iter.by_ref().next_if(|c| *c != ' ')))
                    .collect();
                iter.next();
                // println!("{s}");
                stack.push_top(Sexpr::Atom(A::Symbol(s.to_string())));
            }
        }
    }
    Sexpr::List(Vec::new())
}
