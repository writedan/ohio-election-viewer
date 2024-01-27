# Ohio Election Visualizer

This tool takes precinct data reported by the Ohio Secretary of State and projects the results onto county subdivisions as determined by the Census Bureau. Thus, one can view election data on the level of township or municipality as the case may be, a much finer level of detail than is available to the average person without significant costs of money or time.

The website is built in HTML and Javascript; there is a Python script which converts from the requisite workbooks to a database used by the website.

## Building Election Data

There is a folder simply denotd `elections`. Everything necessary is located in there, but I have only uploaded that data which I manually created out of licensing concerns. Thankfully, the other requisite components are readily available online.

Within the `elections` folder is a series of other folders, each with a given year. These are election years. For example, the `elections/2022` folder contains a `general-precinct-conversion.xlsx` file. This is the file I manually wrought; it consists of a list of precincts by county, to which I have attached the correspondong county subdivision FIPS code. The `general-` here means that it is a general election.

Besides this, a couple of other files are necessary. First, you need to create a folder called `general-maps`. This will contain the shapefile data for Ohio as of the 2022 general elections. There are two sub-folders you need to create: `county` and `municipality`, respectively containing the shapefiles for the counties and county subdivisions of Ohio.

The county shapefiles are available from the [Ohio Department of Transportation](https://gis.dot.state.oh.us/tims/Data/Download) under `Boundaries`, select `County` and then export. Be sure to rename each file to merely `county`, e.g. `county.shp`. 

The Census Bureau [maintains shapefiles](https://catalog.data.gov/dataset/tiger-line-shapefile-2018-state-ohio-current-county-subdivision-state-based) for various years. Ensure you have the shapefiles current for the given election, and add to the `municipality` folder, aptly renaming the files therein as well.

Finally, you need a `subdivision-codes.xlsx` file. This is [also available](https://www.census.gov/library/reference/code-lists/ansi.html#cousub) from the Census Bureau.

Now with all this, the only thing missing is the election data itself. These are available from the [Ohio Secretary of State](https://www.ohiosos.gov/elections/election-results-and-data/). The expected format only goes back to the 2012 general election. Be sure you are downloading the `by Precinct` election files. This file requires some slight modification, namely that the `Master` sheet should be deleted. If you want to do multiple elections at once, e.g. when there are both statewide offices and ballot issues, you can name these, e.g. `general-election-1.xlsx`, `general-election-2.xlsx`, and so forth.

Make sure your database is equipped. It must be named `elections.db`. If you want a fresh version, the `backup.db.schema` has all the queries necessary to set it up.

You may then run `python3 election-converter.py <election type> <election year>`. `Election type` refers to the different elections within one year, e.g. a general, special, primary. The prefix of your files must correspond to this. The script may take several minutes to execute, because of the breadth of data.