-- Add migration script here
CREATE TABLE image (
    img_id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    hash TEXT NOT NULL,
    size INTEGER NOT NULL CHECK (size > 0),
    date_added DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    date_updated DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT unique_path UNIQUE (path)
);

CREATE INDEX idx_image_added ON image (date_added DESC);
CREATE INDEX idx_image_updated ON image (date_updated DESC);
