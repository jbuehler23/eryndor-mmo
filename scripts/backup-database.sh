#!/bin/bash
# Database backup script

BACKUP_DIR="/var/backups/eryndor"
DB_PATH="/var/lib/eryndor/eryndor.db"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p "$BACKUP_DIR"

if [ -f "$DB_PATH" ]; then
    echo "Backing up database..."
    cp "$DB_PATH" "$BACKUP_DIR/eryndor.db.$TIMESTAMP"
    echo "Backup created: $BACKUP_DIR/eryndor.db.$TIMESTAMP"

    # Keep last 7 days
    find "$BACKUP_DIR" -name "eryndor.db.*" -mtime +7 -delete
    echo "Old backups cleaned up"
else
    echo "Database not found at $DB_PATH"
fi
