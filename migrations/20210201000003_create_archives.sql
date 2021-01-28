CREATE TABLE IF NOT EXISTS archives (
    id UUID NOT NULL PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    repo_id VARCHAR NOT NULL,
    created_date TIMESTAMP NOT NULL,
    directory_id UUID,
    CONSTRAINT fk_directories
        FOREIGN KEY (directory_id)
        REFERENCES directories(id)
        ON DELETE SET NULL
);