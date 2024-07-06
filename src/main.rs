use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(subcommand_value_name = "module")]
struct Cli {
    #[command(subcommand)]
    module: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Extracts municipal codes from a Shapefile map and creates a table.")]
    ExtractMunicipalCodes {
        #[arg(long, value_name = "year", help = "The year of the election, which must exist as a directory in elections/")]
        year: String,

        #[arg(long, value_name = "type", help = "The type of the election, which must exist as a directory in elections/<year>/")]
        r#type: String,

        #[arg(long, value_name = "map", help = "Path to a directory containing the shapefile. The name of each file therein MUST match the name of the directory.")]
        mapPath: String
    },

    #[command(about = "Load election results into the database.")]
    ElectionConverter {
        #[arg(long, value_name = "year", help = "The year of the election, which must exist as a directory in elections/")]
        year: String,

        #[arg(long, value_name = "type", help = "The type of the election, which must exist as a directory in elections/<year>/")]
        r#type: String,

        #[arg(long, value_name = "name", help = "If you wish to provide a different name than the default \"<year> <type> Election\"")]
        name: Option<String>
    },

    #[command(about = "Launch a HTTP server on the given address.")]
    RunServer {
        #[arg(long, value_name = "bind address", help = "The IP and port to launch the server on.")]
        bind_to: String,
    },
}

fn main() {
    use colored::Colorize;
    use std::path::PathBuf;

    let cli = Cli::parse();
    match &cli.module {
        Commands::ExtractMunicipalCodes { year, r#type, mapPath } => {
            let mapPath: PathBuf = mapPath.into();

            let generalName = match mapPath.file_name() {
                Some(name) => name.to_string_lossy(),
                None => {
                    emit(Log::Error(format!("Failed to get filename for path {}", mapPath.display())));
                    return;
                }
            }.to_string();

            let mut reader = match shapefile::Reader::from_path(mapPath.join(generalName.clone()).with_extension("shp")) {
                Ok(reader) => reader,
                Err(why) => {
                    emit(Log::Error(format!("Failed to open shapefile: {}", why.to_string().underline())));
                    emit(Log::Error(format!("Ensure this file exists: {}", mapPath.join(generalName.clone()).with_extension("shp").display().to_string().underline())));
                    emit(Log::Error(format!("Ensure this file exists: {}", mapPath.join(generalName.clone()).with_extension("dbf").display().to_string().underline())));
                    return;
                }
            };

            let workbook_uri: PathBuf = ["elections", &year, &r#type, "municipal-codes.xlsx"].iter().collect();
            let mut workbook = rust_xlsxwriter::Workbook::new();

            let mut sheet = workbook.add_worksheet();

            let mut reserve = TwoKeyMap::<String, String, String>::new();

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
                        reserve.insert(county.to_string(), name.to_string(), fips.to_string());
                    },

                    _ => {
                        emit(Log::Error("some field failed to be set"));
                        emit(Log::Info("there is likely an error in your shapefile map"));
                        return
                    }
                }
            }

            for (idx, (county, name, fips)) in reserve.iter_ordered().enumerate() {
                let idx = idx as u32;
                sheet.write(idx, 0, name);
                sheet.write(idx, 1, county);
                sheet.write(idx, 2, fips);
            }

            workbook.save(workbook_uri.clone()).unwrap();
            println!("{} Successfully wrote and saved {}", "Finished!".green().bold(), workbook_uri.display().to_string().underline());
        },

        Commands::ElectionConverter { year, r#type, name } => {
            let mut r#type = r#type.to_owned();

            if let Some(t) = r#type.get_mut(0..1) {
                t.make_ascii_uppercase();
            }

            let r#type = r#type; // shadow as non-mutable

            let name = match name {
                Some(name) => name.to_owned(),
                None => format!("{} {} Election", year, r#type)
            };

            emit(Log::Info(format!("Adding {} to the election index.", name.underline())));
            emit(Log::Info("If this was not the desired name, delete it from the database and run again with the --name argument set."));

            let workbook_uri: PathBuf = ["elections", &year, &r#type.to_lowercase()].iter().collect();
            let precinct_wb = workbook_uri.join("precinct-conversions.xlsx"); // precinct to city/township FIPS; county abbreviation to county names
            let municipal_wb = workbook_uri.join("municipal-codes.xlsx"); // fips codes to municipal names (and canonical county)
            let results_wbs = find_matching_files(&workbook_uri, "election-results");
            if !precinct_wb.exists() {
                emit(Log::Error(format!("File does not exist: {}", precinct_wb.display().to_string().underline())));
                return;
            }

            if !municipal_wb.exists() {
                emit(Log::Error(format!("File does not exist: {}", municipal_wb.display().to_string().underline())));
                return;
            }

            if results_wbs.len() == 0 {
                emit(Log::Error(format!("No result workbooks found in {}", workbook_uri.display().to_string().underline())));
                return;
            }
        },

        Commands::RunServer { bind_to } => {
            emit(Log::Error("not yet implemented"));
        }
    }
}

struct TwoKeyMap<K1, K2, V> {
    items: Vec<(K1, K2, V)>
}

impl<K1: Ord + Clone, K2: Ord + Clone, V: Clone> TwoKeyMap<K1, K2, V> {
    fn iter_ordered(&self) -> impl Iterator<Item = (K1, K2, V)> {
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

impl<K1: std::cmp::PartialEq, K2: std::cmp::PartialEq, V> TwoKeyMap<K1, K2, V> {
    fn insert(&mut self, key1: K1, key2: K2, value: V) {
        self.items.push((key1, key2, value));
    }

    fn get(&self, key1: &K1, key2: &K2) -> Option<&V> {
        self.items.iter()
            .find(|(k1, k2, _)| k1 == key1 && k2 == key2)
            .map(|(_, _, v)| v)
    }

    fn new() -> TwoKeyMap<K1, K2, V> {
        TwoKeyMap::<K1, K2, V> {
            items: Vec::new()
        }
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
            } else if path.is_dir() {
                results.extend(find_matching_files(&path, pattern));
            }
        }
    }

    results
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