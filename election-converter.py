# This file converts the workbooks provided by the Ohio Secretary of State from preinct-level results into aggregates municipality-level results and pushes them into the database.
# Election workbooks are not included with this repository.

# Dependencies: pandas, openpyxl

import pandas as pd
import openpyxl
import sqlite3
import sys

# workbook = openpyxl.load_workbook(sys.argv[1])
# conn = sqlite3.connect('elections.sqlite3')
# cursor = conn.cursor()

# contents = workbook['Contents']['A1'].value.split(', ')
# electionInfo = contents[1].split(' Official')[0]
# electionDate = contents[0]

# print(electionInfo)
# print(electionDate)


# cursor.execute(f"INSERT INTO info(name, date) VALUES('{electionInfo}', '{electionDate}')")


electionYear = int(sys.argv[1])
electionType = sys.argv[2]

database = sqlite3.connect('backup.db')
cursor = database.cursor()

subdivisionWorkbookUri = 'elections/' + str(electionYear) + '/' + electionType + '-subdivision-codes.xlsx'
print("Load:", subdivisionWorkbookUri)
subdivisionWorkbook = openpyxl.load_workbook(subdivisionWorkbookUri)

precinctWorkbookUri = 'elections/' + str(electionYear) + '/' + electionType + '-precinct-conversion.xlsx'
print("Load:", precinctWorkbookUri)
precinctWorkbook = openpyxl.load_workbook(precinctWorkbookUri)

electionWorkbookUri = 'elections/' + str(electionYear) + '/' + electionType + '-election.xlsx'
print("Load:", electionWorkbookUri)
electionWorkbook = openpyxl.load_workbook(electionWorkbookUri)




# First, we will extract the election date and fill in the election data
electionDate = electionWorkbook['Contents']['A1'].value.split(', ')[0]
electionName = electionWorkbook['Contents']['A1'].value.split(', ')[1].split('\n')[0]

print(f"Adding to election index: {electionName}")

cursor.execute(f"INSERT INTO election_info(name, date, year) VALUES('{electionName}', '{electionDate}', '{electionYear}')")

# we need to get our new id
electionInfoId = cursor.lastrowid


# Now we can begin adding counties and municipalities
data = subdivisionWorkbook['Sheet1'].values
columns = next(data) # isolate the first row as columns
df = pd.DataFrame(data, columns=columns)
countyIndex = {}

for idx, some in df.iterrows():
	countyIndex[some['COUNTYNAME']] = some['COUNTYFP']

for idx in countyIndex:
	name = idx
	fips = countyIndex[idx]

	print("Processing county", name)

	cursor.execute(f"INSERT INTO county(name, fips, electionId) VALUES('{name}', '{fips}', '{electionInfoId}')")

	countyId = cursor.lastrowid

	# now here we must begin adding municipalities
	muns = df[df['COUNTYFP'] == fips]
	for idx, m in muns.iterrows():
		mName = m['COUSUBNAME']
		mCode = m['COUSUBFP']

		print("\tProcessing municipality", mName)
		
		cursor.execute(f"INSERT INTO municipality(name, fips, countyId) VALUES('{mName}', '{mCode}', '{countyId}')")

# now we can setup the precincts with their conversions
data = precinctWorkbook['Sheet1'].values
columns = next(data)
df = pd.DataFrame(data, columns=columns)

for idx, precinct in df.iterrows():
	countyName = precinct['COUNTYNAME'] + ' County'
	municipalCode = precinct['MUNICIPALFIPS']
	precinctName = precinct['PRECINCTNAME']

	print("Processing precinct", precinctName,"of",countyName)

	cursor.execute(f"SELECT * FROM municipalities WHERE electionId='{electionInfoId}' AND countyName='{countyName}' AND municipalCode='{municipalCode}'")

	for m in cursor.fetchall():
		cursor.execute(f"INSERT INTO precinct(name, municipalId) VALUES('{precinctName}', '{m[0]}')")

print("Converting election data.")
