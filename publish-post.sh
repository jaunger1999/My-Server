#!/bin/bash

# Define the database file
DATABASE=$1

# Define the title and file path
TITLE=$2

# Read the content of the text file into a variable
TEXT_CONTENT=$(<$3)

# Get the current Unix timestamp
CURRENT_TIMESTAMP=$(date +%s)

# Create the SQL query to insert a new entry
SQL_QUERY="INSERT INTO entries (date_entered, date_last_edited, title, text_format) VALUES ($CURRENT_TIMESTAMP, $CURRENT_TIMESTAMP, '$TITLE', '$TEXT_CONTENT');"

# Run the query using sqlite3
sqlite3 $DATABASE $SQL_QUERY

# Print a success message
echo "Entry added successfully."
