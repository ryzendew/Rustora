#!/bin/bash

LOG_DIR="$HOME/.fedoraforge"
mkdir -p "$LOG_DIR"

echo "🔍 Monitoring Flatpak logs in: $LOG_DIR"
echo "📝 Waiting for new log files..."
echo ""

# Function to display log file
display_log() {
    local log_file="$1"
    echo ""
    echo "═══════════════════════════════════════════════════════════"
    echo "📄 NEW LOG FILE: $(basename "$log_file")"
    echo "═══════════════════════════════════════════════════════════"
    cat "$log_file"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
}

# Watch for new files
if command -v inotifywait &> /dev/null; then
    inotifywait -m "$LOG_DIR" -e create --format '%w%f' | while read -r file; do
        if [[ "$file" == *.log ]]; then
            sleep 0.5  # Wait a bit for file to be fully written
            display_log "$file"
        fi
    done
else
    # Fallback: poll for new files
    echo "⚠️  inotifywait not available, using polling mode..."
    LAST_FILES=$(ls -1 "$LOG_DIR"/*.log 2>/dev/null | wc -l)
    while true; do
        CURRENT_FILES=$(ls -1 "$LOG_DIR"/*.log 2>/dev/null | wc -l)
        if [ "$CURRENT_FILES" -gt "$LAST_FILES" ]; then
            NEWEST=$(ls -t "$LOG_DIR"/*.log 2>/dev/null | head -1)
            if [ -n "$NEWEST" ]; then
                display_log "$NEWEST"
            fi
            LAST_FILES=$CURRENT_FILES
        fi
        sleep 1
    done
fi

