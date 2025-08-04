CREATE TABLE tags (
    id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tag_name TEXT UNIQUE NOT NULL
);

CREATE TABLE session_tags (
    session_id INTEGER REFERENCES sessions (id) ON DELETE CASCADE,
    tag_id INTEGER REFERENCES tags (id) ON DELETE CASCADE,
    PRIMARY KEY (session_id, tag_id)
)