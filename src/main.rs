mod converter;
use clap::{Parser, Subcommand};


#[derive(Parser)]
#[command(subcommand_value_name = "module")]
struct Cli {
    #[command(subcommand)]
    module: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Initializes the database.")]
    InitDatabase {
        #[arg(long, help = "If the database already exists, remove it and re-initialize.", action = clap::ArgAction::SetTrue)]
        force: bool
    },

    #[command(about = "Extracts municipal codes from a Shapefile map and creates a table.")]
    ExtractMunicipalCodes {
        #[arg(long, value_name = "directory", help = "The name of the direction in elections/ containing preinct-results.xlsx.\nFor example, 2022/general.")]
        election_path: String,

        #[arg(long, value_name = "directory", help = "Path to a directory containing the shapefile. The name of each file therein MUST match the name of the directory.")]
        map_path: String
    },

    #[command(about = "Load election results into the database.", name = "convert-election")]
    ElectionConverter {
        #[arg(long, value_name = "directory", help = "The name of the direction in elections/ containing preinct-results.xlsx.\nFor example, 2022/general.")]
        election_path: String,

        #[arg(long, value_name = "name", help = "The name of the election. Uses derived value otherwise.")]
        name: Option<String>
    },

    #[command(about = "Launch a HTTP server on the given address.")]
    RunServer {
        #[arg(long, value_name = "bind address", help = "The IP and port to launch the server on.")]
        bind_to: String,
    },
}

fn raw_query<S: Into<String> + Copy>(conn: &rusqlite::Connection, q: S) {
    query(conn, q, ())
}

fn query<S: Into<String> + Copy, P: rusqlite::Params>(conn: &rusqlite::Connection, query: S, params: P) {
    use colored::Colorize;

    match conn.execute(query.into().as_str(), params) {
        Ok(_) => {},
        Err(why) => {
            emit(Log::Error(format!("failed to run query: {}", query.into().underline())));
            emit(Log::Error(why.to_string()));
        }
    }
}

fn main() {
    use colored::Colorize;
    use std::path::PathBuf;

    let cli = Cli::parse();
    match &cli.module {
        Commands::InitDatabase { force } => {
            use rusqlite::Connection;

            let path = PathBuf::from("elections.db");
            if path.exists() && !force {
                emit(Log::Error("database already initialized"));
                emit(Log::Info("run with --force argument to override"));
                return;
            }

            match std::fs::remove_file(path) {
                Ok(_) => {},
                Err(why) => return emit(Log::Error(format!("failed to remove {}: {}", "elections.db".underline(), why.to_string().underline())))
            };

            let mut conn = match Connection::open("elections.db") {
                Ok(conn) => conn,
                Err(why) => return emit(Log::Error(format!("unable to establish connection: {}", why.to_string().underline())))
            };

            let conn = conn.savepoint().unwrap();
            match conn.execute_batch("
                CREATE TABLE election_info(id integer primary key autoincrement, name text, date date, map text);
                CREATE TABLE county(id integer primary key autoincrement, name text, electionId integer, foreign key (electionId) references election_info(id));
                CREATE TABLE municipality(id integer primary key autoincrement, name text, fips text, electionId integer, foreign key (electionId) references election_info(id));
                CREATE TABLE precinct(id integer primary key autoincrement, name text, municipalId integer, countyId integer, foreign key (municipalId) references municipality(id), foreign key (countyId) references county(id));
                CREATE TABLE office_category(id integer primary key autoincrement, name text, electionId integer, foreign key (electionId) references election_info(id));
                CREATE TABLE office_election(id integer primary key autoincrement, name text, categoryId integer, foreign key (categoryId) references office_category(id));
                CREATE TABLE candidate(id integer primary key autoincrement, name text, officeId integer, foreign key (officeId) references office_election(id));
                CREATE TABLE result(id integer primary key autoincrement, votes integer, candidateId integer, precinctId integer, foreign key (candidateId) references candidate(id), foreign key (precinctId) references precinct(id));

                CREATE VIEW state_results as select r.officeId, sum(r.votes) as votes, r.candidateId, r.candidateName from county_results r group by r.candidateId;
                CREATE VIEW municipal_results as select m.id, r.officeId, sum(r.votes) as votes, r.candidateId, r.candidateName, m.name as municipalName, m.fips as municipalCode, m.electionId from precinct_results r join municipality m on r.municipalId = m.id group by r.candidateId, m.id;
                CREATE VIEW county_results as select c.id, r.officeId, sum(r.votes) as votes, r.candidateId, r.candidateName, c.name as countyName from precinct_results r join county c on r.countyId = c.id group by r.candidateId, c.id;
                CREATE VIEW precinct_results as select r.id, c.officeId, r.votes, r.candidateId, c.name as candidateName, p.id as precinctId, p.name as precinctName, p.municipalId, p.countyId from result r inner join candidate c on r.candidateId = c.id inner join precinct p on r.precinctId = p.id;
            ") {
                Ok(_) => {},
                Err(why) => {
                    return emit(Log::Error(format!("Failed to initialized database: {}", why.to_string().underline())));
                }
            };
            conn.commit().unwrap();
            println!("{} Database initialized.", "Success!".green().bold());
        },

        Commands::ExtractMunicipalCodes { election_path, map_path } => {
            let map_path: PathBuf = map_path.into();

            let general_name = match map_path.file_name() {
                Some(name) => name.to_string_lossy(),
                None => {
                    emit(Log::Error(format!("Failed to get filename for path {}", map_path.display().to_string().underline())));
                    return;
                }
            }.to_string();

            let mut reader = match shapefile::Reader::from_path(map_path.join(general_name.clone()).with_extension("shp")) {
                Ok(reader) => reader,
                Err(why) => {
                    emit(Log::Error(format!("Failed to open shapefile: {}", why.to_string().underline())));
                    emit(Log::Error(format!("Ensure this file exists: {}", map_path.join(general_name.clone()).with_extension("shp").display().to_string().underline())));
                    emit(Log::Error(format!("Ensure this file exists: {}", map_path.join(general_name.clone()).with_extension("dbf").display().to_string().underline())));
                    return;
                }
            };

            let workbook_uri: PathBuf = [&election_path, "municipal-codes.xlsx"].iter().collect();
            let mut workbook = rust_xlsxwriter::Workbook::new();

            let mut sheet = workbook.add_worksheet();

            let mut reserve = TwoKeyMap::<String, String, String, String>::new();

            for (idx, shape_record) in reader.iter_shapes_and_records().enumerate() {
                let (shape, record) = match shape_record {
                    Ok((shape, record)) => (shape, record),
                    Err(why) => {
                        emit(Log::Error(format!("{}", why)));
                        return;
                    }
                };

                let name = match record.get("name") {
                    Some(dbase::FieldValue::Character(s)) => s,
                    _ => {
                        emit(Log::Error(format!("Failed to get field {}.", "name".underline())));
                        emit(Log::Error(format!("{:#?}", record)));
                        return;
                    }
                };

                let fips = match record.get("fips") {
                    Some(dbase::FieldValue::Character(s)) => s,
                    _ => {
                        emit(Log::Error(format!("Failed to get field {}.", "fips".underline())));
                        emit(Log::Error(format!("{:#?}", record)));
                        return;
                    }
                };

                let county = match record.get("county") {
                    Some(dbase::FieldValue::Character(s)) => s,
                    _ => {
                        emit(Log::Error(format!("Failed to get field {}.", "county".underline())));
                        emit(Log::Error(format!("{:#?}", record)));
                        return;
                    }
                };

                match (name, fips, county) {
                    (Some(name), Some(fips), Some(county)) => {
                        let r#type = if name.ends_with("(Township)") { "township" } else { "city/village" };
                        let name = if r#type == "township" { name.split(" (Township)").collect::<Vec<&str>>()[0] } else { name.split(" (City)").collect::<Vec<&str>>()[0] };

                        reserve.insert(county.to_string(), name.to_string(), fips.to_string(), r#type.to_string());
                    },

                    _ => {
                        emit(Log::Error("some field failed to be set"));
                        emit(Log::Info("there is likely an error in your shapefile map"));
                        return
                    }
                }
            }

            for (idx, (county, name, fips, r#type)) in reserve.iter_ordered().enumerate() {
                let idx = idx as u32;
                sheet.write(idx, 0, name).unwrap();
                sheet.write(idx, 2, county).unwrap();
                sheet.write(idx, 3, fips).unwrap();
                sheet.write(idx, 1, r#type).unwrap();
            }

            workbook.save(workbook_uri.clone()).unwrap();
            println!("{} Successfully wrote and saved {}", "Finished!".green().bold(), workbook_uri.display().to_string().underline());
        },

        Commands::ElectionConverter { election_path, name } => converter::run(election_path.to_owned(), name),

        Commands::RunServer { bind_to } => {
            emit(Log::Error("not yet implemented"));
        }
    }
}

struct TwoKeyMap<K1, K2, V1, V2> {
    items: Vec<(K1, K2, V1, V2)>
}

impl<K1: Ord + Clone, K2: Ord + Clone, V1: Clone, V2: Clone> TwoKeyMap<K1, K2, V1, V2> {
    fn iter_ordered(&self) -> impl Iterator<Item = (K1, K2, V1, V2)> {
        let mut items = self.items.clone();
        items.sort_by(|a, b| {
            match a.0.cmp(&b.0) {
                std::cmp::Ordering::Equal => a.1.cmp(&b.1),
                other => other,
            }
        });

        items.into_iter()
    }
}

impl<K1, K2, V1, V2> TwoKeyMap<K1, K2, V1, V2> {
    fn insert(&mut self, key1: K1, key2: K2, value1: V1, value2: V2) {
        self.items.push((key1, key2, value1, value2));
    }

    fn new() -> TwoKeyMap<K1, K2, V1, V2> {
        TwoKeyMap::<K1, K2, V1, V2> {
            items: Vec::new()
        }
    }
}

pub enum Log<S: Into<String>> {
    Info(S),
    Warning(S),
    Error(S)
}

pub fn emit<S: Into<String>>(log: Log<S>) {
    use colored::Colorize;
    use crate::Log::*;

    match log {
        Info(s) => println!("{}: {}", "note".cyan().bold(), s.into()),
        Warning(s) => println!("{}: {}", "warning".yellow().bold(), s.into()),
        Error(s) => println!("{}: {}", "error".red().bold(), s.into().bright_red())
    }
}