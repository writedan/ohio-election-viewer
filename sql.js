async function loadDatabase(){
  let SQL;
  let db;

  SQL = await initSqlJs({
    locateFile: file => `https://cdnjs.cloudflare.com/ajax/libs/sql.js/1.10.2/sql-wasm.wasm`
  });
    
  await fetch('http://localhost:3000/elections.db')
    .then(response => response.arrayBuffer())
    .then(buffer => {
      db = new SQL.Database(new Uint8Array(buffer));
    })
    .catch(error => {
      console.error('Error fetching or loading the database:', error);
    });

  return db;
}

function stmt(sqlStatement, parameters = []) {
  const statement = db.prepare(sqlStatement);

  try {
    statement.bind(parameters);

    const resultSet = [];
    while (statement.step()) {
      resultSet.push(statement.getAsObject());
    }

    return resultSet;
  } finally {
    statement.free();
  }
}