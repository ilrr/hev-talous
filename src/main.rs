use std::{
    env,
    fs::{self, read_to_string},
};

use calamine::{open_workbook, Data, Reader, Xlsx};
use tk::parse_tk;
use tlk::{Sexpr, A};

use crate::tlk::parse;

mod tk;
mod tlk;

fn row_to_sexpr(row: &[Data], event_index: &mut i32) -> Sexpr {
    let mut d = vec![Sexpr::Atom(A::Symbol("date".to_string()))];
    d.append(
        &mut row[0]
            .to_string()
            .split('.')
            .rev()
            .map(|v| Sexpr::Atom(A::Number(v.parse::<i32>().expect("Huono päivämäärä"))))
            .collect::<Vec<Sexpr>>(),
    );
    let date = Sexpr::List(d);

    let column_6 = row[6].to_string();
    let description = (if column_6.len() > 0 {
        format!(
            "{} / {} / {}",
            row[4].to_string(),
            row[5].to_string(),
            row[6].to_string()
        )
    } else {
        format!("{} / {}", row[4].to_string(), row[5].to_string())
    })
    .replace("\n", r#"\n"#);
    let account = match row[3].to_string().as_str() {
        "Palvelumaksut" => 3210,
        s => s[..4].parse().expect("Excelissä on jotain häikkää..."),
    };
    let amount: i32 = (row[7]
        .to_string()
        .parse::<f32>()
        .expect("Excelissä on jotain häikkää...")
        * 100.0) as i32;
    let i = event_index.to_owned();
    *event_index += 1;
    Sexpr::List(vec![
        Sexpr::Atom(A::Symbol("event".to_string())),
        Sexpr::Atom(A::Number(i)),
        date,
        Sexpr::Atom(A::String(description)),
        Sexpr::List(vec![
            Sexpr::List(vec![
                Sexpr::Atom(A::Number(1130)),
                Sexpr::List(vec![
                    Sexpr::Atom(A::Symbol("money".to_string())),
                    Sexpr::Atom(A::Number(amount)),
                ]),
            ]),
            Sexpr::List(vec![
                Sexpr::Atom(A::Number(account)),
                Sexpr::List(vec![
                    Sexpr::Atom(A::Symbol("money".to_string())),
                    Sexpr::Atom(A::Number(-amount)),
                ]),
            ]),
        ]),
    ])
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let workbook = &args[1];
    let input_ledger = &args[2];
    let output_ledger = &args[3];

    let start_date = Sexpr::List(vec![
        Sexpr::Atom(A::Symbol("date".to_string())),
        Sexpr::Atom(A::Number(2023)),
        Sexpr::Atom(A::Number(1)),
        Sexpr::Atom(A::Number(1)),
    ]);
    let end_date = Sexpr::List(vec![
        Sexpr::Atom(A::Symbol("date".to_string())),
        Sexpr::Atom(A::Number(2023)),
        Sexpr::Atom(A::Number(12)),
        Sexpr::Atom(A::Number(31)),
    ]);
    // let (a, t) =
    //     parse_tk("-1 A\n 1 B\n 2 C\n 30 D\n  31 Da\n  32 Db\n-1 N\n      33 m\n-1 -1".to_string());
    // let (a, t) =
    //       parse_tk("-1 A\n 1 B\n 2 C\n-1 N".to_string());  println!("{a}\n{t}");
    //

    let mut events: Vec<Sexpr>;
    let mut account_map = Sexpr::List(Vec::new());
    let input_ledger_is_tlk = input_ledger.ends_with(".tlk");
    if !input_ledger_is_tlk {
        let account = read_to_string(input_ledger).expect("Tiedoston luku ei onnistunut");
        let (acc, evs) = parse_tk(account);
        account_map = acc;
        match evs {
            Sexpr::List(l) => events = l,
            _ => panic!(),
        }
    } else {
        events = Vec::new();
    }

    let mut w: Xlsx<_> =
        open_workbook(workbook).expect(&format!("Virheellinen Excel-polku: {workbook}"));
    let mut event_index: i32 = 0;
    // let mut events: Vec<Sexpr> = Vec::new();

    if let Ok(range) = w.worksheet_range("Päiväkirja") {
        range
            .rows()
            .skip(6)
            // .take(1)
            .for_each(|r| events.push(row_to_sexpr(r, &mut event_index)));
    }

    let mut a: Sexpr;
    if input_ledger_is_tlk {
        let account = read_to_string(input_ledger).expect("Tiedoston luku ei onnistunut.");
        a = parse(account);
        a.mutate_5_5(Sexpr::List(events));
    } else {
        a = Sexpr::List(vec![
            Sexpr::Atom(A::Symbol("identity".to_string())),
            Sexpr::Atom(A::String("Tappio".to_string())),
            Sexpr::Atom(A::Symbol("version".to_string())),
            Sexpr::Atom(A::String("versio 0.22".to_string())),
            Sexpr::Atom(A::Symbol("finances".to_string())),
            Sexpr::List(vec![
                Sexpr::Atom(A::Symbol("fiscal-year".to_string())),
                Sexpr::Atom(A::String("HEV 2023".to_string())),
                start_date,
                end_date,
                account_map,
                Sexpr::List(events),
            ]),
        ])
    }

    fs::write(output_ledger, a.to_string()).expect("Tiedoston kirjoittaminen epäonnistui");
}
