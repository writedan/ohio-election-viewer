# This file converts the workbooks provided by the Ohio Secretary of State from preinct-level results into aggregates municipality-level results and pushes them into the database.
# Election workbooks are not included with this repository.

# Dependencies: pandas, openpyxl

import pandas as pd
import openpyxl
import sqlite3
import sys

workbook = openpyxl.load_workbook(sys.argv[1])
conn = sqlite3.connect('elections.sqlite3')
cursor = conn.cursor()

contents = workbook['Contents']['A1'].value.split(', ')
electionInfo = contents[1].split(' Official')[0]
electionDate = contents[0]

print(electionInfo)
print(electionDate)


cursor.execute(f"INSERT INTO info(name, date) VALUES('{electionInfo}', '{electionDate}')")