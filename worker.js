// This file is to launch worker threads which materialize view tables
// That is, we have municipal_results and state_results tables which are very time-consuming to compute
// Thus, at the beginning of the application runtime we would like to convert these into tables
// That is to say, materialize the view

// Each worker maintains a separate instance of the database, so queries must be made out to the worker

let db;

importScripts('https://cdnjs.cloudflare.com/ajax/libs/sql.js/1.10.2/sql-wasm.js')
importScripts('./sql.js')

self.onmessage = async function(event) {
	const data = event.data;
	
	if (data.action == 'init') {
		db = await loadDatabase();
		self.postMessage({state:'connected'})
	}

	if (data.action == 'query') {
		r = stmt(data.query, data.params)
		self.postMessage({result:r, id:data.id})
	}
}