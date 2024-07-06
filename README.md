# Building Election Data

## Map Generation

Due to the nature of copyright law I felt there was little I could include directly in the repository itself. It is necessary therefore to do some manual arrangement if you wish to run a local copy.

First, we need to establish the maps. The program assumes that we are using the [Ohio Department of Transportation GIS maps](https://gis.dot.state.oh.us/tims/Data/Download). You must download all of the township, city, and county maps. Import all these as vectors into QGIS (or another tool, but I used QGIS).

You may need to fix the geometries of the city layer by running the "Fix geometries" tool. If so, remove the original `REFER_CITY` layer the rename the resulting `Fixed geometries` to `REFER_CITY`. Then, create a difference layer using the "Difference" tool with `REFER_TOWNSHIP` as input and `REFER_CITY` as overlay. Finally, use the "Merge vector layers" tool to merge `Difference` with `REFER_TOWNSHIP`, creating a `Merged` layer.

You will then need to load the `qgis-layer-editor.py` file into the Python console and execute it. This will create a `municipals` layer, which you will then export as an ESRI Shapefile. **Note that the name of the directory must correspond to the name of the Shapefile.** If you named it "map.shp" (and "map.dbf" and so forth), these must be in a directory named "map".

## Data Extraction

Now that we have generated a map we need to extract its data and populate the database with it. For each election there are essentially two parts, viz. the election results itself (by precinct) and the metadata over those precincts which we use to group them into cities and counties. In this step we will create a conversion table for FIPS codes to municipal names, which data is not within the precinct results workbooks.

Fortunately this step is simple and is built into the program: `ohio-election-visualizer extract-municipal-codes --year=<YEAR> --type=<TYPE> --map-path=PATH_TO_MAP_DIRECTORY`. This will create a `municipal-codes.xlsx` in the `elections` directory for the given year and type of election.