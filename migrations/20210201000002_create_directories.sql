CREATE TABLE IF NOT EXISTS directories (
    id UUID NOT NULL PRIMARY KEY,
    path VARCHAR UNIQUE NOT NULL,
    blake3_hash VARCHAR NOT NULL,
    root_directory_id UUID NOT NULL,
    CONSTRAINT fk_root_directories
        FOREIGN KEY (root_directory_id)
        REFERENCES root_directories(id)
        ON DELETE CASCADE
);
