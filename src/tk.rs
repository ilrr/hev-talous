use std::{
    borrow::{Borrow, BorrowMut},
    fmt::Debug,
};

use crate::tlk::{Sexpr, A};

struct Account {
    number: i32,
    name: String,
    children: Vec<Account>,
}

impl Account {
    fn to_sexpr(&mut self) -> Sexpr {
        let n = self.name.clone();
        Sexpr::List(vec![
            Sexpr::Atom(A::Symbol("account".to_string())),
            Sexpr::Atom(A::Number(self.number)),
            Sexpr::Atom(A::String(n)),
            Sexpr::List(self.children.iter_mut().map(|a| a.to_sexpr()).collect()),
        ])
    }
    fn push_child(&mut self, v: Account) {
        self.children.push(v);
    }
    fn append_children(&mut self, v: &mut Vec<Account>) {
        self.children.append(v);
    }
}

impl Debug for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {:?})", self.number, self.name, self.children)
    }
}

fn init_account(number: i32, name: String) -> Account {
    Account {
        number,
        name,
        children: Vec::new(),
    }
}

fn account_vec_to_sexpr(v: &mut Vec<Account>) -> Sexpr {
    Sexpr::List(v.iter_mut().map(|a| a.to_sexpr()).collect())
}

pub fn parse_tk(tk: String) -> (Sexpr, Sexpr) {
    let mut account_map_vec: Vec<Sexpr> = Vec::new();
    let mut events_vec: Vec<Sexpr> = Vec::new();
    let mut indent_stack: Vec<i32> = vec![];
    let mut account_stack: Vec<Vec<Account>> = vec![];
    let mut lines: Vec<&str> = tk.lines().rev().collect();
    // for line in tk.lines() {
    while lines.len() > 0 {
        let line = lines.pop().unwrap();
        let mut line_iter = line.trim_end().chars().peekable();
        let indent = {
            let mut l: i32 = 0;
            while line_iter.peek().unwrap().is_whitespace() {
                l += 1;
                line_iter.next();
            }
            l
        };
        let account_n: i32 = line_iter
            .borrow_mut()
            .take_while(|c| *c == '-' || c.is_ascii_digit())
            .collect::<String>()
            .parse()
            .unwrap();
        let account_name: String = match line_iter.clone().last() {
            Some(c) => match c {
                '€' | '$' => {
                    let temp = line_iter.collect::<String>();
                    let (v1, v2) = temp.rsplit_once(' ').unwrap();
                    let mut num_str = v2.to_string();
                    num_str.pop();
                    let amount = (num_str.replace(",", ".").parse::<f32>().unwrap() * 100.0) as i32;
                    events_vec.push(Sexpr::List(vec![
                        Sexpr::Atom(A::Number(account_n)),
                        Sexpr::List(vec![
                            Sexpr::Atom(A::Symbol("money".to_string())),
                            Sexpr::Atom(A::Number(amount)),
                        ]),
                    ]));
                    v1.to_string()
                }
                _ => line_iter.collect::<String>(),
            },
            _ => panic!(),
        };

        // let new_sexpr = Sexpr::List(vec![
        //     Sexpr::Atom(A::Symbol("account".to_string())),
        //     Sexpr::Atom(A::Number(account_n)),
        //     Sexpr::Atom(A::String(account_name)),
        //     Sexpr::List(Vec::new()),
        // ]);
        let new_account = init_account(account_n, account_name);
        let top_indent = indent_stack.last();

        // println!("{indent}, '{line}'");
        match top_indent {
            None => {
                account_stack.push(vec![new_account]);
                indent_stack.push(indent);
            }
            Some(&top_i) => {
                // println!("({})", top_i > indent);
                if top_i == indent {
                    // account_stack.last_mut().unwrap().last_mut().unwrap().push_child(new_account);
                    account_stack.last_mut().unwrap().push(new_account);
                } else if top_i < indent {
                    account_stack.push(Vec::new());
                    account_stack.last_mut().unwrap().push(new_account);
                    indent_stack.push(indent);
                } else {
                    // println!("!");
                    let mut i = indent_stack.pop().unwrap();
                    // println!("{}", i >= indent);
                    while i > indent {
                        // println!(" > {account_stack:?}\n   {indent_stack:?} {i}");
                        let mut top = account_stack.pop().unwrap();
                        // println!(" > > {top:?}");
                        account_stack
                            .last_mut()
                            .unwrap()
                            .last_mut()
                            .unwrap()
                            .append_children(top.as_mut());
                        // account_stack.last_mut().unwrap().last_mut().unwrap().push_child(top);
                        i = indent_stack.pop().unwrap();
                    }
                   // account_stack.push(vec![new_account]);
                   lines.push(line);
                    indent_stack.push(indent);
                }
            }
        }


        // println!("{account_stack:?} {indent_stack:?}\n");
    }
        let mut i = indent_stack.pop().unwrap();
        while i > 0 {
            // println!(" > {account_stack:?}\n   {indent_stack:?} {i}");
            let mut top = account_stack.pop().unwrap();
            // println!(" > > {top:?}");
            account_stack
                .last_mut()
                .unwrap()
                .last_mut()
                .unwrap()
                .append_children(top.as_mut());
            i = indent_stack.pop().unwrap();
        }
    // println!(". {account_stack:?}");
    let mut a: Vec<Account> = account_stack.into_iter().flatten().collect();
    // let accounts_vec: Vec<Sexpr> = account_stack
    //     .iter_mut()
    //     .map(|a| account_vec_to_sexpr(a))
    //     .collect();
    let accounts_vec = a.iter_mut().map(|a| a.to_sexpr()).collect();
    // println!(". {accounts_vec:?}");
    (Sexpr::List(accounts_vec), Sexpr::List(events_vec))
}
