use std::{
    env::{self, args},
    fs::{self, read_to_string, File},
    io::{LineWriter, Write},
};

use calamine::{open_workbook, Data, Reader, Xlsx};
use tk::parse_tk;
use tlk::{Sexpr, A};

use crate::tlk::parse;

mod tk;
mod tlk;

const OFFSET: usize = 1;

fn date_str_to_sexpr(s: String) -> Sexpr {
    let mut d = vec![Sexpr::Atom(A::Symbol("date".to_string()))];
    d.append(
        &mut s
            .to_string()
            .split('.')
            .rev()
            .map(|v| Sexpr::Atom(A::Number(v.parse::<i32>().expect("Huono päivämäärä"))))
            .collect::<Vec<Sexpr>>(),
    );
    Sexpr::List(d)
}

fn row_to_sexpr(row: &[Data], event_index: &mut i32) -> Sexpr {
    let date = date_str_to_sexpr(row[0 + OFFSET].to_string());

    let column_6 = row[6 + OFFSET].to_string();
    let description = (if column_6.len() > 0 {
        format!(
            "{} / {} / {}",
            row[4 + OFFSET].to_string(),
            row[5 + OFFSET].to_string(),
            row[6 + OFFSET].to_string()
        )
    } else {
        format!(
            "{} / {}",
            row[4 + OFFSET].to_string(),
            row[5 + OFFSET].to_string()
        )
    })
    .replace("\n", r#"\n"#);
    let account = match row[3 + OFFSET].to_string().as_str() {
        "Palvelumaksut" => 3210,
        s => s[..4].parse().expect("Excelissä on jotain häikkää..."),
    };
    let amount: i32 = (row[7 + OFFSET]
        .to_string()
        .parse::<f64>()
        .expect("Excelissä on jotain häikkää...")
        * 100.0)
        .round() as i32;
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

fn row_to_tampio(
    row: &[Data],
    prev_row: Option<&[Data]>,
    /* next_row:Option<&[Data]>, */ event_index: &mut i32,
) -> String {
    let date = row[0 + OFFSET].to_string();
    let c4 = row[4 + OFFSET].to_string();
    let c5 = row[5 + OFFSET].to_string();
    let c6 = row[6 + OFFSET].to_string();
    let mut description = (if c6.len() > 0 {
        format!("{} / {} / {}", c4, c5, c6)
    } else {
        format!("{} / {}", c4, c5)
    })
    .replace("\n", "¶");
    description = if description.contains('"') {
        format!("»{description}»")
    } else {
        format!("\"{description}\"")
    };
    let account = match row[3 + OFFSET].to_string().as_str() {
        "Palvelumaksut" => 3210,
        "Luokittelemattomat" => 8000,
        s => s[..4].parse().unwrap_or(8000), // .expect("Excelissä on jotain häikkää..."),
    };
    // println!("{}", row[7 + OFFSET]);
    let amount: f64 = -row[7 + OFFSET]
        .to_string()
        .parse::<f64>()
        .expect("Excelissä on jotain häikkää...");
    // let i = event_index.to_owned();
    if let Some(prev_row) = prev_row {
        if !(prev_row[4 + OFFSET].to_string() == c4
            && prev_row[6 + OFFSET].to_string() == c6
            && !c6.is_empty())
        {
            *event_index += 1;
        }
    }
    format!("{date} {description} H{event_index}\n    {account}: {amount:.2}\n")
}

fn main() -> std::io::Result<()> {
    let args = env::args().collect::<Vec<String>>();
    let workbook = &args[1];
    let input_ledger = &args[2];

    if args.len() == 3 {
        let mut w: Xlsx<_> =
            open_workbook(workbook).expect(&format!("Virheellinen Excel-polku: {workbook}"));
        let mut event_index: i32 = 1;
        let f = File::create(input_ledger)?;
        let mut f = LineWriter::new(f);

        if let Ok(range) = w.worksheet_range("Päiväkirja") {
            let mut prev_row = None;
            write!(&mut f, "holvi = 1130\nAUTO holvi [\n")?;
            for row in range.rows().skip(6) {
                f.write_all(row_to_tampio(row, prev_row, &mut event_index).as_bytes())?;
                prev_row = Some(row);
            }
            write!(&mut f, "]")?;
        }
        return f.flush();
        // return Ok(());
    }

    let output_ledger = &args[3];

    /*let start_date = Sexpr::List(vec![
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
    ]); */
    let mut start_date = Sexpr::Atom(A::Symbol("no-date".to_string()));
    let mut end_date = Sexpr::Atom(A::Symbol("no-date".to_string()));
    let mut title = Sexpr::Atom(A::String("".to_string()));

    let mut events: Vec<Sexpr>;
    let mut account_map = Sexpr::List(Vec::new());
    let input_ledger_is_tlk = input_ledger.ends_with(".tlk");
    if !input_ledger_is_tlk {
        let ledger = read_to_string(input_ledger).expect("Tiedoston luku ei onnistunut");
        let (header, account) = ledger.split_once("\n\n").unwrap();
        let (acc, evs) = parse_tk(account.to_string());
        account_map = acc;
        let mut header_vec = header.split("\n");
        title = Sexpr::Atom(A::String(
            header_vec.next().expect("Epäkelpo tiedosto").to_string(),
        ));
        start_date = date_str_to_sexpr(header_vec.next().expect("Epäkelpo tiedosto").to_string());
        end_date = date_str_to_sexpr(header_vec.next().expect("Epäkelpo tiedosto").to_string());

        let opening_date = start_date.clone();
        events = vec![Sexpr::List(vec![
            Sexpr::Atom(A::Symbol("event".to_string())),
            Sexpr::Atom(A::Number(0)),
            opening_date,
            Sexpr::Atom(A::String("Tilikauden avaus".to_string())),
            evs,
        ])];
    } else {
        events = Vec::new();
    }

    let mut w: Xlsx<_> =
        open_workbook(workbook).expect(&format!("Virheellinen Excel-polku: {workbook}"));
    let mut event_index: i32 = 1;

    if let Ok(range) = w.worksheet_range("Päiväkirja") {
        range
            .rows()
            .skip(6)
            // .take(0)
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
                title,
                start_date,
                end_date,
                {
                    let mut am =
                        Sexpr::List(vec![Sexpr::Atom(A::Symbol("account-map".to_string()))]);
                    am.append(account_map);
                    am
                },
                Sexpr::List(events),
            ]),
        ])
    }

    fs::write(output_ledger, a.to_string()) //.expect("Tiedoston kirjoittaminen epäonnistui");
}
