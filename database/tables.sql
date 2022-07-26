CREATE TABLE files(
    file_id INT NOT NULL AUTO_INCREMENT,
    file_name VARCHAR(255) NOT NULL,
    full_path VARCHAR(255) NOT NULL UNIQUE,
    file_size INT NOT NULL,
    parent_dir_id  INT NOT NULL,
    PRIMARY KEY (file_id),
    CONSTRAINT fk_dirs
    FOREIGN KEY (parent_dir_id)
    REFERENCES dirs(dir_id)
        ON UPDATE CASCADE
        ON DELETE CASCADE
)

CREATE TABLE dirs(
    dir_id INT,
    dir_name VARCHAR(255) NOT NULL,
    full_path VARCHAR(255) NOT NULL UNIQUE,
    parent_dir_id  INT NOT NULL,
    PRIMARY KEY (dir_id),
    CONSTRAINT fk_dirs
    FOREIGN KEY (parent_dir_id)
    REFERENCES dirs(dir_id)
        ON UPDATE CASCADE
        ON DELETE CASCADE
)
