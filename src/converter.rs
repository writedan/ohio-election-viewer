use crate::Log;

pub fn run(election_path: String, name: &Option<String>) {
    use std::collections::HashMap;
    use rusqlite::Connection;
    use std::path::PathBuf;
    use colored::Colorize;
    use chrono::Datelike;
    use crate::emit;
    use std::fs::File;
    use std::io::Write;
    use calamine::{Xlsx, Reader};

    let workbook_uri: PathBuf = election_path.clone().into();
    let precinct_wb = workbook_uri.join("precinct-conversions.xlsx"); // precinct to city/township FIPS; county abbreviation to county names
    let municipal_wb = workbook_uri.join("municipal-codes.xlsx"); // fips codes to municipal names (and canonical county)
    let results_wbs = find_matching_files(&workbook_uri, "election-results");
    if !precinct_wb.exists() {
        return emit(Log::Error(format!("File does not exist: {}", precinct_wb.display().to_string().underline())));
    }

    if !municipal_wb.exists() {
        return emit(Log::Error(format!("File does not exist: {}", municipal_wb.display().to_string().underline())));
    }

    if results_wbs.len() == 0 {
        return emit(Log::Error(format!("No result workbooks found in {}", workbook_uri.display().to_string().underline())));
    }

    // first we need to identify which municipalities to exclude from the final map
    // as well as setup our tables
    let mut output_file = match File::create("map.filter") {
        Ok(file) => file,
        Err(why) => return emit(Log::Error(format!("unable to open {}: {}", "map.filter".underline(), why.to_string().underline())))
    };

    if !PathBuf::from("elections.db").exists() {
        emit(Log::Error(format!("file does not exist: {}", "elections.db".underline())));
        return emit(Log::Info(format!("run the {} module", "init".underline())));
    }

    let conn = match Connection::open("elections.db") {
        Ok(conn) => conn,
        Err(why) => return emit(Log::Error(format!("unable to establish connection: {}", why.to_string().underline())))
    };

    let mut conn = match Connection::open("elections.db") {
        Ok(conn) => conn,
        Err(why) => return emit(Log::Error(format!("unable to establish connection: {}", why.to_string().underline())))
    };

    let mut conn = conn.savepoint().unwrap();

    let mut precinct_wb = calamine::open_workbook_auto(precinct_wb).unwrap();
    let mut county_wb = precinct_wb.worksheet_range("counties").unwrap();
    let mut precinct_wb = precinct_wb.worksheet_range("precincts").unwrap();
    let mut municipal_wb = calamine::open_workbook_auto(municipal_wb).unwrap().worksheet_range("Sheet1").unwrap();
    let mut results_wbs: Vec<_> = results_wbs.iter().map(|wb| {
        print!("Opening workbook {}", wb.display().to_string().underline());
        std::io::stdout().flush().expect("Unable to flush stdout.");
        let mut wb = calamine::open_workbook_auto(wb).unwrap();
        println!(" {}", "done".green());
        let mut sheets = Vec::new();
        for x in wb.sheet_names() {
            print!("Loading sheet {}", x.underline());
            std::io::stdout().flush().expect("Unable to flush stdout.");
            if let "Contents" | "Master" = x.as_str() {
                println!(" {}", "skipped".yellow());
                continue; // we pass over this one because all its data is kept in the other sheets
            }

            sheets.push(wb.worksheet_range(&x).unwrap());

            println!(" {}", "done".green());
        }

        sheets
    }).flatten().collect();

    let contents = results_wbs[0].get_value((0, 0)).expect("Cell A1 must at least begin with the date of the election.").to_string();
    let (date, name) = match extract_date_and_remainder(contents.as_str()) {
        Ok((date, title)) => (date, title.split("Official").collect::<Vec<_>>()[0].trim()),
        Err(why) => {
            emit(Log::Error(format!("Failed to get date from cell AI: {}", why.to_string())));
            return;
        }
    };

    let name = format!("{} {}", date.year(), name);

    emit(Log::Info(format!("Adding {} to the election index.", name.underline())));
    emit(Log::Info("If this was not the desired name, delete it from the database and run again with the --name argument set."));
    
    let map_path: PathBuf = PathBuf::from(election_path).join("map");
    conn.execute("INSERT INTO election_info(name, date, map) VALUES(?1, ?2, ?3)", (name, date, map_path.display().to_string())).unwrap();
    conn.commit().unwrap();

    let mut county_abbr_lookup: HashMap<String, String> = HashMap::new(); // abbr -> name
    for row in 0..county_wb.get_size().0 {
        let row = row as u32;
        if let (Some(abbr), Some(name)) = (county_wb.get_value((row, 0)), county_wb.get_value((row, 1))) {
            county_abbr_lookup.insert(abbr.to_string(), name.to_string());
        } else {
            emit(Log::Warning(format!("Unable to resolve county on row={}", row)));
        }
    }
}

fn extract_date_and_remainder(input: &str) -> Result<(chrono::NaiveDate, &str), chrono::ParseError> {
    use chrono::{NaiveDate};
    use chrono::format::{ParseError, ParseErrorKind};

    let format = "%B %d, %Y";
    
    if let Some(comma_pos) = input.find(',') {
        let date_str = &input[..comma_pos + 6];

        let date = NaiveDate::parse_from_str(date_str, format)?;

        let remainder = &input[comma_pos + 7..];

        Ok((date, remainder))
    } else {
        match NaiveDate::parse_from_str("November 5", format) {
            Ok(_) => panic!("This statement can never be reached."),
            Err(why) => Err(why)
        } // trigger ParseError::NotEnough
    }
}

fn find_matching_files(dir: &std::path::Path, pattern: &str) -> Vec<std::path::PathBuf> {
    use std::fs;

    let mut results = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    if let Some(name_str) = file_name.to_str() {
                        if name_str.starts_with(pattern) && name_str.ends_with(".xlsx") {
                            results.push(path.clone());
                        }
                    }
                }
            }
        }
    }

    results
}