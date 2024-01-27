# This file converts the workbooks provided by the Ohio Secretary of State from preinct-level results into aggregates municipality-level results and pushes them into the database.
# Election workbooks are not included with this repository.

# Dependencies: pandas, openpyxl

import pandas as pd
import openpyxl
import sqlite3
import sys

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

<<<<<<< HEAD
cursor.execute(f"INSERT INTO election_info(name, date, year) VALUES(?, ?, ?)", (electionName, electionDate, electionYear))
=======
cursor.execute(f"INSERT INTO election_info(name, date, year) VALUES(?, ?, ?)", electionName, electionDate, electionYear)
>>>>>>> refs/remotes/origin/main

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

<<<<<<< HEAD
	cursor.execute(f"INSERT INTO county(name, fips, electionId) VALUES(?, ?, ?)", (name, fips, electionInfoId))
=======
	cursor.execute(f"INSERT INTO county(name, fips, electionId) VALUES(?, ?, ?)", name, fips, electionInfoId)
>>>>>>> refs/remotes/origin/main

	countyId = cursor.lastrowid

	# now here we must begin adding municipalities
	muns = df[df['COUNTYFP'] == fips]
	for idx, m in muns.iterrows():
		mName = m['COUSUBNAME']
		mCode = m['COUSUBFP']

		print("\tProcessing municipality", mName)
		
<<<<<<< HEAD
		cursor.execute(f"INSERT INTO municipality(name, fips, countyId) VALUES(?, ?, ?)", (mName, mCode, countyId))
=======
		cursor.execute(f"INSERT INTO municipality(name, fips, countyId) VALUES(?, ?, ?)", mName, mCode, countyId)
>>>>>>> refs/remotes/origin/main

# now we can setup the precincts with their conversions
data = precinctWorkbook['Sheet1'].values
columns = next(data)
df = pd.DataFrame(data, columns=columns)

for idx, precinct in df.iterrows():
	countyName = precinct['COUNTYNAME'] + ' County'
	municipalCode = precinct['MUNICIPALFIPS']
	precinctName = precinct['PRECINCTNAME']

	print("Processing precinct", precinctName,"of",countyName)

<<<<<<< HEAD
	cursor.execute(f"SELECT * FROM municipalities WHERE electionId=? AND countyName=? AND municipalCode=?", (electionInfoId, countyName, municipalCode))

	for m in cursor.fetchall():
		cursor.execute(f"INSERT INTO precinct(name, municipalId) VALUES(?, ?)", (precinctName, m[0]))
=======
	cursor.execute(f"SELECT * FROM municipalities WHERE electionId=? AND countyName=? AND municipalCode=?", electionInfoId, countyName, municipalCode)

	for m in cursor.fetchall():
		cursor.execute(f"INSERT INTO precinct(name, municipalId) VALUES(?, ?)", precinctName, m[0])
>>>>>>> refs/remotes/origin/main

for worksheet in electionWorkbook.sheetnames:
	if (worksheet == 'Contents' or worksheet == 'Master'):
		continue

<<<<<<< HEAD
	cursor.execute(f"INSERT INTO office_category(name, electionId) VALUES (?, ?)", (worksheet, electionInfoId))
=======
	cursor.execute(f"INSERT INTO office_category(name, electionId) VALUES (?, ?)", worksheet, electionInfoId)
>>>>>>> refs/remotes/origin/main
	officeCategoryId = cursor.lastrowid

	print("Processing election category", worksheet)

	worksheet = electionWorkbook[worksheet]

	# offices = {}
	lastOffice = ''
	officeElectionId = -1

	for columnIdx in range(9, worksheet.max_column): # 9th column idx = 'I'
		val = worksheet.cell(row=1, column=columnIdx).value
		if (val is not None):
			val = val.strip().replace('\n', ' - ')
			print("\tProcessing office", val)
			# we have a new office
			lastOffice = val
<<<<<<< HEAD
			cursor.execute(f"INSERT INTO office_election(name, categoryId) VALUES(?, ?)", (lastOffice, officeCategoryId))
=======
			cursor.execute(f"INSERT INTO office_election(name, categoryId) VALUES(?, ?)", lastOffice, officeCategoryId)
>>>>>>> refs/remotes/origin/main
			officeElectionId = cursor.lastrowid

		candidateName = worksheet.cell(row=2, column=columnIdx).value
		candidateName.replace('\n', ' - ')
		print("\t\tProcessing candidate", candidateName)

		if (candidateName[-1] == '*'):
			print("\t\t\tWrite-in candidates are not collated. Rejected.")
			continue

<<<<<<< HEAD
		cursor.execute(f"INSERT INTO candidate(name, officeId) VALUES(?, ?)", (candidateName, officeElectionId))
=======
		cursor.execute(f"INSERT INTO candidate(name, officeId) VALUES(?, ?)", candidateName, officeElectionId)
>>>>>>> refs/remotes/origin/main
		candidateId = cursor.lastrowid

		for rowIdx in range(5, worksheet.max_row):
			countyName = worksheet.cell(row=rowIdx,column=1).value + ' County'
			precinctName = worksheet.cell(row=rowIdx,column=2).value
			candidateVotes = worksheet.cell(row=rowIdx,column=columnIdx).value
<<<<<<< HEAD
			cursor.execute(f"SELECT id FROM precincts WHERE countyName=? AND precinctName=? AND electionId=?", (countyName, precinctName, electionInfoId))

			for p in cursor.fetchall():
				precinctId = p[0]
				cursor.execute(f"INSERT INTO office_result(votes, candidateId, precinctId) VALUES(?, ?, '?)", (candidateVotes, candidateId, precinctId))
=======
			cursor.execute(f"SELECT id FROM precincts WHERE countyName=? AND precinctName=? AND electionId=?", countyName, precinctName, electionInfoId)

			for p in cursor.fetchall():
				precinctId = p[0]
				cursor.execute(f"INSERT INTO office_result(votes, candidateId, precinctId) VALUES(?, ?, '?)", candidateVotes, candidateId, precinctId)
>>>>>>> refs/remotes/origin/main
