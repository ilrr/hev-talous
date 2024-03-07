use std::{
    env,
    fs::{self, read_to_string},
};

use calamine::{open_workbook, Data, Reader, Xlsx};
use tlk::{Sexpr, A};

use crate::tlk::parse;

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

    let mut w: Xlsx<_> =
        open_workbook(workbook).expect(&format!("Virheellinen Excel-polku: {workbook}"));
    let mut event_index: i32 = 0;
    let mut events: Vec<Sexpr> = Vec::new();

    if let Ok(range) = w.worksheet_range("Päiväkirja") {
        range
            .rows()
            .skip(6)
            // .take(1)
            .for_each(|r| events.push(row_to_sexpr(r, &mut event_index)));
    }

    let account = read_to_string(input_ledger).expect("Tiedoston luku ei onnistunut.");
    let mut a = parse(account);

    a.mutate_5_5(Sexpr::List(events));

    fs::write(output_ledger, a.to_string()).expect("Tiedoston kirjoittaminen epäonnistui");
}
