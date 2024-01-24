# This file converts the subdivisions in the "Ohio Subdivision Codes" worksheet into the "city" table of the database.

# Dependencies: pandas, openpyxl

import pandas as pd
import sqlite3

df = pd.read_excel('Ohio Subdivision Codes.xlsx', header=0)
conn = sqlite3.connect('elections.sqlite3')
cursor = conn.cursor()

for idx, data in df.iterrows():
	countyFips = str(data['COUNTYFP'])
	if len(countyFips) < 3:
		countyFips = ("0" * (3 - len(countyFips))) + countyFips

	stmt = f"SELECT id FROM county WHERE fips='{countyFips}'"
	cursor.execute(stmt)
	for r in cursor.fetchall():
		countyCode = r[0]
		stmt = f"INSERT INTO city(name, county, fips) VALUES('{data['COUSUBNAME']}', '{countyCode}', '{data['COUSUBFP']}')"
		cursor.execute(stmt)

conn.commit()
conn.close()