# Installation

	git clone https://github.com/writedan/ohio-election-visualizer
	cd ohio-election-visualizer
	cargo build --release
	cargo install --path .
	oev-tool init-database

These commands will configure the enviroment to begin building election data. This tool has a number of dependencies. These are the last versions which have been verified to produce a build:

	rouille = "3.6.2"
	calamine = "0.25.0"
	rusqlite = "0.31.0"
	clap = "4.5.9"
	serde = "1.0.204"
	colored = "2.1.0"
	shapefile = "0.6.0"
	dbase = "0.5.0"
	rust_xlsxwriter = "0.69.0"
	chrono = "0.4.38"

Cargo is configured to allow any version which according to Semver ought to be future-compatible. If this is not the case, restore to one of the versions given above.

# Building Election Data

1. Download the [Ohio Department of Transportation GIS maps](https://gis.dot.state.oh.us/tims/Data/Download) or other appropriate source. You will need minimally a county subdivision map and county map. To use the full suite of tools, download from ODOT their city, township, and county maps.

2. Open those maps in QGIS or other GIS manipulator. Create a difference layer using the "Difference" tool with `REFER_TOWNSHIP` as input and `REFER_CITY` as the overlay. Then use the "Merge vector layers" tool to combine `Difference` and `REFER_TOWNSHIP` into one `Merged` layer. **Note that** the `REFER_CITY` layer, in my experience, has been invalid. You will then need to use the "Fix geometries" tool to repair it, then delete the original `REFER_CITY` and rename the repaired `Fix geometries` layer to `REFER_CITY`

3. Load the `qgis-layer-editor.py` script contained in this repository into the QGIS Python console and execute it. This will create a `municipals` layer. Save this layer in a `maps` directory as an ESRI Shapefile, but the name of the direcory ***must*** correspond to the name of the files. For example, if the Shapefile is named `2024-ohio.shp`, the directory must also be named `2024-ohio`.

4. Save the `REFER_COUNTY` layer in `maps/county-map`, naming the Shapefile `county.shp` (regardless of the directive in the previous step).

5. Download election results from the [Ohio Secretary of State](https://www.ohiosos.gov/elections/election-results-and-data/) and place the `.xlsx` file in a directory in `elections/`. Rename the file to `election-results.xlsx`. If you wish to include multiple such files (for example, state issues are reported separately from statewide offices despite concurrent elections), you can give them sequential numbers, e.g. `election-results-1.xlsx`, `elections-results-2.xlsx`, and so forth.

6. Run the command `oev-tool extract-municipal-codes --election-path=elections/<DIRECTORY CONTAINING ELECTION RESULTS> --map-path=maps/<DIRECTORY CONTAINING EXPORTED municipals LAYER>`. This will create two files: `municipal-codes.xlsx` and `precinct-conversions.xlsx`, the latter which is broken into two worksheets, `precincts` and `counties`. You must now manually assign each precinct in `precincts` to one or more municipal codes from the other workbook (using each additional column as one code). If multiple municipalities are assigned to the same precinct they will be merged into one. You must also assign each 3-character county code to its full name in the `counties` sheet. You may also rename the resulting municipalities/townships by editing `municipal-codes.xlsx`.

7. Once each precinct has been assigned to one or more municipalities, run the command `oev-tool import-election --election-path=elections/<DIRECTORY CONTAINING ELECTION RESULTS>`. This will first add the election data into the database, but also create two files in the project base: `map-filter.temp` and `map-merge.temp`. 

8. Again lload `REFER_TOWNSHIP` and `REFER_CITY` into QGIS along with the generated `municipals` layer saved in `maps`. Now load `qgis-layer-finalizer.py` into the Python console and execute it. This will create a new layer, `finalized_municipals` which you must save as `map/map.shp` in the directory containing `election-results.xlsx` and the rest. 

9. You then have successfully setup at least one election. Run `oev-tool run-server --bind-to=<IP:PORT>` to launch a HTTP server.