#[derive(serde::Serialize)]
pub struct Election {
	name: String,
	date: chrono::NaiveDate,
	map_path: String,
	id: usize
}

#[derive(serde::Serialize)]
pub struct OfficeCategory {
	name: String,
	id: usize
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct ElectionResult {
	name: String,
	votes: usize
}

#[derive(serde::Serialize, Debug)]
pub struct SumElectionResult {
	total_votes: usize,
	candidates: Vec<ElectionResult>
}

#[derive(serde::Serialize, Debug)]
pub struct County {
	name: String,
	id: usize,
	election: SumElectionResult
}

#[derive(serde::Serialize, Debug)]
pub struct Error {
	error: String
}

impl From<rusqlite::Error> for Error {
	fn from(err: rusqlite::Error) -> Self { 
		Error {
			error: err.to_string()
		}
	}
}

impl From<String> for Error {
	fn from(err: String) -> Self { 
		Error {
			error: err
		}
	}
}

impl From<std::num::ParseIntError> for Error {
	fn from(err: std::num::ParseIntError) -> Self { 
		Error {
			error: err.to_string()
		}
	}
}

type Result = std::result::Result<rouille::Response, Error>;

pub fn election_manifest() -> Result {
	let mut conn = rusqlite::Connection::open("./elections.db")?;
	let conn = conn.savepoint()?;

	let mut stmt = conn.prepare("SELECT * FROM election_info ORDER BY date DESC")?;
	let result_vec = stmt.query_map([], |row| {
		Ok(Election {
			name: row.get(1)?,
			date: row.get(2)?,
			map_path: row.get(3)?,
			id: row.get(0)?
		})
	})?.collect::<Vec<std::result::Result<Election, rusqlite::Error>>>();

	let mut elections = Vec::new();
	for election in result_vec.iter() {
		match election {
			Ok(e) => elections.push(e),
			Err(why) => return Err(why.to_string().into())
		}
	}

	Ok(rouille::Response::json(&elections))
}

pub fn election_categories(id: usize) -> Result {
	let mut conn = rusqlite::Connection::open("./elections.db")?;
	let conn = conn.savepoint()?;

	let mut stmt = conn.prepare("SELECT * FROM office_category WHERE electionId=?1")?;
	let result_vec = stmt.query_map([id], |row| {
		Ok(OfficeCategory {
			name: row.get(1)?,
			id: row.get(0)?
		})
	})?.collect::<Vec<std::result::Result<OfficeCategory, rusqlite::Error>>>();

	let mut categories = Vec::new();
	for category in result_vec.iter() {
		match category {
			Ok(category) => categories.push(category),
			Err(why) => return Err(why.to_string().into())
		}
	}

	Ok(rouille::Response::json(&categories))
}

pub fn category_offices(id: usize) -> Result {
	let mut conn = rusqlite::Connection::open("./elections.db")?;
	let conn = conn.savepoint()?;

	let mut stmt = conn.prepare("SELECT * FROM office_election WHERE categoryId=?1")?;
	let result_vec = stmt.query_map([id], |row| {
		Ok(OfficeCategory {
			name: row.get(1)?,
			id: row.get(0)?
		})
	})?.collect::<Vec<std::result::Result<OfficeCategory, rusqlite::Error>>>();

	let mut categories = Vec::new();
	for category in result_vec.iter() {
		match category {
			Ok(category) => categories.push(category),
			Err(why) => return Err(why.to_string().into())
		}
	}

	Ok(rouille::Response::json(&categories))
}

pub fn state_results(id: usize) -> Result {
	let mut conn = rusqlite::Connection::open("./elections.db")?;
	let conn = conn.savepoint()?;

	let mut stmt = conn.prepare("SELECT sum(votes) as totalVotes from indexed_state_results WHERE officeId=?1")?;
	let results_vec = stmt.query_map([id], |row| {
		row.get(0)
	})?.collect::<Vec<std::result::Result<String, rusqlite::Error>>>();
	if results_vec.len() > 1 {
		return Err(Error {
			error: format!("Expected 1 row, got {}", results_vec.len())
		});
	}

	let Ok(total_votes) = &results_vec[0] else { return Err(Error { error: format!("Failed to get total_votes for officeId={}: {:#?}", id, results_vec) }) };

	let mut stmt = conn.prepare("SELECT * FROM indexed_state_results WHERE officeId=?1")?;
	let results_vec = stmt.query_map([id], |row| {
		Ok(ElectionResult {
			votes: row.get(1)?,
			name: row.get(3)?
		})
	})?.collect::<Vec<std::result::Result<ElectionResult, rusqlite::Error>>>();

	let mut res = Vec::new();
	for candidate in results_vec.iter() {
		match candidate {
			Ok(candidate) => res.push(candidate.clone()),
			Err(why) => return Err(why.to_string().into())
		}
	}

	Ok(rouille::Response::json(&SumElectionResult {
		total_votes: total_votes.parse::<usize>()?,
		candidates: res
	}))
}

fn county_results_query(county_id: usize, office_id: usize) -> std::result::Result<SumElectionResult, Error> {
	let mut conn = rusqlite::Connection::open("./elections.db")?;
	let conn = conn.savepoint()?;

	let mut stmt = conn.prepare("SELECT name FROM county WHERE id=?1")?;
	let results_vec = stmt.query_map([county_id], |row| {
		row.get(0)
	})?.collect::<Vec<std::result::Result<String, rusqlite::Error>>>();
	if results_vec.len() > 1 {
		return Err(Error {
			error: format!("Expected 1 row, got {}", results_vec.len())
		});
	}

	let Ok(name) = &results_vec[0] else { return Err(Error { error: format!("Failed to get county name") }) };

	let mut stmt = conn.prepare("SELECT sum(votes) as totalVotes from indexed_county_results WHERE officeId=?1 AND countyName=?2")?;
	let results_vec = stmt.query_map([&office_id.to_string(), name], |row| {
		row.get(0)
	})?.collect::<Vec<std::result::Result<usize, rusqlite::Error>>>();
	if results_vec.len() > 1 {
		return Err(Error {
			error: format!("Expected 1 row, got {}", results_vec.len())
		});
	}

	let Ok(total_votes) = &results_vec[0] else { return Err(Error { error: format!("Failed to get total_votes for countyId={} and officeId={}: {:?}", county_id, office_id, results_vec) }) };

	let mut stmt = conn.prepare("SELECT candidateName, votes FROM indexed_county_results WHERE countyName=?1 and officeId=?2")?;
	let results_vec = stmt.query_map([name, &office_id.to_string()], |row| {
		Ok(ElectionResult {
			name: row.get(0)?,
			votes: row.get(1)?
		})
	})?.collect::<Vec<std::result::Result<ElectionResult, rusqlite::Error>>>();

	let mut res = Vec::new();
	for ele in results_vec.iter() {
		match ele {
			Ok(ele) => res.push(ele.clone()),
			Err(why) => return Err(why.to_string().into())
		}
	}

	Ok(SumElectionResult {
		total_votes: *total_votes,
		candidates: res
	})
}

pub fn county_results(county_id: usize, office_id: usize) -> Result {
	Ok(rouille::Response::json(&county_results_query(county_id, office_id)?))
}

pub fn counties(election_id: usize, office_id: usize) -> Result {
	let mut conn = rusqlite::Connection::open("./elections.db")?;
	let conn = conn.savepoint()?;

	let mut stmt = conn.prepare("SELECT name, id FROM county WHERE electionId=?1")?;
	let results_vec = stmt.query_map([election_id], |row| {
		Ok(OfficeCategory {
			name: row.get(0)?,
			id: row.get(1)?
		})
	})?.collect::<Vec<std::result::Result<OfficeCategory, rusqlite::Error>>>();

	let mut res: std::collections::HashMap<String, County> = std::collections::HashMap::new();
	for ele in results_vec.iter() {
		match ele {
			Ok(ele) => res.insert(ele.name.clone(), County {
				name: ele.name.clone(),
				id: ele.id,
				election: match county_results_query(ele.id, office_id) {
					Ok(e) => e,
					Err(why) => continue
				}
			}),
			Err(why) => None // skip
		};
	}

	Ok(rouille::Response::json(&res))
}

pub fn unpack(r: Result) -> rouille::Response {
	match r {
		Ok(r) => r,
		Err(err) => rouille::Response::json(&err)
	}
}