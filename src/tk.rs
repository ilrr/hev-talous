use std::borrow::{Borrow, BorrowMut};

use crate::tlk::{Sexpr, A};

pub fn parse_tk(tk: String) -> (Sexpr, Sexpr) {
    let mut account_map_vec: Vec<Sexpr> = Vec::new();
    let mut events_vec: Vec<Sexpr> = Vec::new();
    let mut indent_stack: Vec<i32> = vec![];
    let mut account_stack: Vec<Sexpr> = vec![];

    for line in tk.lines() {
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

        let new_sexpr = Sexpr::List(vec![
            Sexpr::Atom(A::Symbol("account".to_string())),
            Sexpr::Atom(A::Number(account_n)),
            Sexpr::Atom(A::String(account_name)),
            Sexpr::List(Vec::new()),
        ]);
        let top_indent = indent_stack.last();

        // println!("A_S: {account_stack:?}, {top_indent:?}");
        match top_indent {
            None => {
                account_stack.push(new_sexpr);
                indent_stack.push(indent);
                // println!("N => {account_stack:?}")
            }
            Some(&ti) => {
                // println!("ti={ti}, indent={indent}");

                /*if indent == 0 {
                    account_stack.push(new_sexpr);
                } else */ if indent > ti {
                    account_stack.push(Sexpr::List(vec![new_sexpr]));
                } else if indent == ti {
                    let mut a = account_stack.pop().unwrap();
                    a.push(new_sexpr);
                    // println!("! {a}");
                    account_stack.push(a);
                    // account_stack.last().unwrap().push(new_sexpr);
                } else {
                    let mut i = indent;
                    // println!("ti {ti}, i {i}");
                    while i < ti {
                        // println!("i {i}");
                        let a = account_stack.pop().unwrap();
                        let mut b = account_stack.pop().unwrap();
                        // println!("] {b}");
                        b.set_last(a);
                        // match b.borrow_mut() {
                        //     Sexpr::List(l) => l.last_mut().unwrap().push_last(a),
                        //     _ => panic!(),
                        // }
                        // b.push_last(a);
                        account_stack.push(b);
                        i = indent_stack.pop().unwrap();
                    }
                    account_stack.push(Sexpr::List(vec![new_sexpr]));
                    // println!("{i}, {account_stack:?}");
                    // if i == 0 {
                    //     account_map_vec.push(account_stack.pop().unwrap());
                    // }
                }
                indent_stack.push(indent);
            }
        }

        // println!("-> {:?} \n", account_stack);
    }
    (Sexpr::List(account_stack), Sexpr::List(events_vec))
}
