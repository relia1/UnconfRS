CREATE TABLE index_markdown (
    id INTEGER GENERATED ALWAYS AS (1) STORED UNIQUE,
    markdown TEXT NOT NULL,
    markdown_converted_to_html TEXT NOT NULL
);
