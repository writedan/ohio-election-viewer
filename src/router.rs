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

#[derive(serde::Serialize)]
pub struct MunicipalHold {
	name: String,
	fips: String
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
pub struct Municipality {
	name: String,
	fips: String,
	election: SumElectionResult
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
	})?.collect::<Vec<std::result::Result<usize, rusqlite::Error>>>();
	if results_vec.len() > 1 {
		return Err(Error {
			error: format!("Expected 1 row, got {}", results_vec.len())
		});
	}

	let Ok(total_votes) = &results_vec[0] else { return Err(Error { error: format!("Failed to get total_votes for officeId={}: {:#?}", id, results_vec) }) };

	let mut stmt = conn.prepare("SELECT * FROM indexed_state_results WHERE officeId=?1 ORDER BY votes DESC")?;
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
		total_votes: *total_votes,
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

	let mut stmt = conn.prepare("SELECT candidateName, votes FROM indexed_county_results WHERE countyName=?1 and officeId=?2 ORDER BY votes DESC")?;
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

fn municipal_results_query(office_id: usize, municipal_fips: &str, conn: &rusqlite::Connection) -> std::result::Result<SumElectionResult, Error> {
    let mut stmt = conn.prepare("SELECT sum(votes) FROM indexed_municipal_results WHERE officeId=?1 AND municipalCode=?2")?;
    let total_votes: usize = stmt.query_row(rusqlite::params![office_id, municipal_fips], |row| row.get(0))?;
    
    let mut stmt = conn.prepare("SELECT candidateName, votes FROM indexed_municipal_results WHERE officeId=?1 AND municipalCode=?2 ORDER BY votes DESC")?;
    let results_vec = stmt.query_map(rusqlite::params![office_id, municipal_fips], |row| {
        Ok(ElectionResult {
            name: row.get(0)?,
            votes: row.get(1)?,
        })
    })?.collect::<SqlResult<Vec<ElectionResult>>>()?;

    Ok(SumElectionResult {
        total_votes,
        candidates: results_vec,
    })
}

pub fn municipal_results(office_id: usize, municipal_fips: String) -> std::result::Result<rouille::Response, Error> {
    let conn = rusqlite::Connection::open("./elections.db")?;
    let result = municipal_results_query(office_id, &municipal_fips, &conn)?;
    Ok(rouille::Response::json(&result))
}

type SqlResult<T> = std::result::Result<T, rusqlite::Error>;

pub fn all_municipalities(office_id: usize) -> Result {
	let conn = rusqlite::Connection::open("./elections.db")?;
    
    let mut stmt = conn.prepare("SELECT m.name, m.fips FROM municipality m JOIN precinct p on m.id = p.municipalId")?;
    let results_vec = stmt.query_map(rusqlite::params![], |row| {
        Ok(MunicipalHold {
            name: row.get(0)?,
            fips: row.get(1)?,
        })
    })?.collect::<SqlResult<Vec<MunicipalHold>>>()?;
    
    let municipal_fips_list: Vec<&str> = results_vec.iter().map(|m| m.fips.as_str()).collect();
    
    // Query all results in batch
    let query_placeholders = municipal_fips_list.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let query = format!(
        "SELECT municipalCode, candidateName, votes, officeId FROM indexed_municipal_results WHERE officeId=?1 AND municipalCode IN ({}) ORDER BY votes DESC",
        query_placeholders
    );
    
    let mut stmt = conn.prepare(&query)?;
    let params: Vec<&dyn rusqlite::ToSql> = vec![&office_id as &dyn rusqlite::ToSql].into_iter().chain(municipal_fips_list.iter().map(|fips| &*fips as &dyn rusqlite::ToSql)).collect();
    let mut results_map: std::collections::HashMap<String, Vec<ElectionResult>> = std::collections::HashMap::new();
    
    let mut rows = stmt.query(params.as_slice())?;
    while let Some(row) = rows.next()? {
        let municipal_code: String = row.get(0)?;
        let candidate_name: String = row.get(1)?;
        let votes: usize = row.get(2)?;
        
        let result = ElectionResult {
            name: candidate_name,
            votes,
        };
        
        results_map.entry(municipal_code).or_insert_with(Vec::new).push(result);
    }
    
    let mut municipalities = std::collections::HashMap::new();
    for m in results_vec.iter() {
        if let Some(election_results) = results_map.get(&m.fips) {
            let total_votes: usize = election_results.iter().map(|r| r.votes).sum();
            municipalities.insert(
                m.fips.clone(),
                Municipality {
                    name: m.name.clone(),
                    fips: m.fips.clone(),
                    election: SumElectionResult {
                        total_votes,
                        candidates: election_results.clone(),
                    },
                },
            );
        }
    }

    Ok(rouille::Response::json(&municipalities))
}

pub fn municipalities(office_id: usize, county_id: usize) -> std::result::Result<rouille::Response, Error> {
    let conn = rusqlite::Connection::open("./elections.db")?;
    
    let mut stmt = conn.prepare("SELECT m.name, m.fips FROM municipality m JOIN precinct p on m.id = p.municipalId WHERE p.countyId=?1")?;
    let results_vec = stmt.query_map(rusqlite::params![county_id], |row| {
        Ok(MunicipalHold {
            name: row.get(0)?,
            fips: row.get(1)?,
        })
    })?.collect::<SqlResult<Vec<MunicipalHold>>>()?;
    
    let municipal_fips_list: Vec<&str> = results_vec.iter().map(|m| m.fips.as_str()).collect();
    
    // Query all results in batch
    let query_placeholders = municipal_fips_list.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let query = format!(
        "SELECT municipalCode, candidateName, votes, officeId FROM indexed_municipal_results WHERE officeId=?1 AND municipalCode IN ({}) ORDER BY votes DESC",
        query_placeholders
    );
    
    let mut stmt = conn.prepare(&query)?;
    let params: Vec<&dyn rusqlite::ToSql> = vec![&office_id as &dyn rusqlite::ToSql].into_iter().chain(municipal_fips_list.iter().map(|fips| &*fips as &dyn rusqlite::ToSql)).collect();
    let mut results_map: std::collections::HashMap<String, Vec<ElectionResult>> = std::collections::HashMap::new();
    
    let mut rows = stmt.query(params.as_slice())?;
    while let Some(row) = rows.next()? {
        let municipal_code: String = row.get(0)?;
        let candidate_name: String = row.get(1)?;
        let votes: usize = row.get(2)?;
        
        let result = ElectionResult {
            name: candidate_name,
            votes,
        };
        
        results_map.entry(municipal_code).or_insert_with(Vec::new).push(result);
    }
    
    let mut municipalities = std::collections::HashMap::new();
    for m in results_vec.iter() {
        if let Some(election_results) = results_map.get(&m.fips) {
            let total_votes: usize = election_results.iter().map(|r| r.votes).sum();
            municipalities.insert(
                m.fips.clone(),
                Municipality {
                    name: m.name.clone(),
                    fips: m.fips.clone(),
                    election: SumElectionResult {
                        total_votes,
                        candidates: election_results.clone(),
                    },
                },
            );
        }
    }

    Ok(rouille::Response::json(&municipalities))
}

pub fn unpack(r: Result) -> rouille::Response {
	match r {
		Ok(r) => r,
		Err(err) => rouille::Response::json(&err)
	}
}