#!/bin/bash

# File to be monitored
FILE_TO_MONITOR="log.temp"

# Temporary file to store the previous state of the file
PREVIOUS_STATE="previous_state.temp"

# Log file to store modification times and changes
LOG_FILE="file_modifications.temp"

# Check if inotifywait is installed
if ! command -v inotifywait &> /dev/null
then
    echo "inotifywait could not be found. Please install inotify-tools."
    exit 1
fi

# Check if diff is installed
if ! command -v diff &> /dev/null
then
    echo "diff could not be found. Please make sure it's installed."
    exit 1
fi

# Initialize the previous state
cp "$FILE_TO_MONITOR" "$PREVIOUS_STATE"

echo "Monitoring $FILE_TO_MONITOR for modifications..."

# Monitor the file for modifications
while inotifywait -e modify "$FILE_TO_MONITOR"; do
    # Get the current timestamp with precision of milliseconds
    CURRENT_TIME=$(date '+%Y-%m-%d %H:%M:%S')$(printf ".%03d" $(($(date +%N) / 1000000)))
    
    # Log the time of modification
    echo "File modified at: $CURRENT_TIME" >> "$LOG_FILE"
    
    # Show and log the changes (diff between previous and current state)
    echo "Changes:" >> "$LOG_FILE"
    diff "$PREVIOUS_STATE" "$FILE_TO_MONITOR" >> "$LOG_FILE"
    
    # Update the previous state
    cp "$FILE_TO_MONITOR" "$PREVIOUS_STATE"
    
    echo "---------------------------------" >> "$LOG_FILE"
done
