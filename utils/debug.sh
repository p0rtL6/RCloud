rm database/files.db database/files.db-shm database/files.db-wal
sqlite3 database/files.db "VACUUM;"
rm -r files
mkdir files
cargo run --release
