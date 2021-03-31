CREATE TABLE tag (
id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
name TEXT(16) NOT NULL,
created_at INTEGER NOT NULL,
updated_at INTEGER,
is_deleted INTEGER DEFAULT 0 NOT NULL,
deleted_at INTEGER,
CONSTRAINT "name" UNIQUE ("name" ASC)
);

CREATE TABLE tag_usage (
id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
blog_id INTEGER NOT NULL,
tag_id INTEGER NOT NULL,
created_at INTEGER NOT NULL,
CONSTRAINT "using_tag_UN" UNIQUE ("blog_id" ASC, "tag_id" ASC),
);
CREATE INDEX blog_id_IDX ON using_tag (blog_id);
CREATE INDEX tag_id_IDX ON using_tag (tag_id);
-- CREATE UNIQUE INDEX blog_id_tag_id_IDX ON using_tag (blog_id,tag_id);

CREATE TABLE post (
id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
title TEXT(64) NOT NULL,
markdown_content TEXT(65535) NOT NULL,
rendered_content TEXT(65535) NOT NULL,
created_at INTEGER NOT NULL,
updated_at INTEGER,
is_deleted INTEGER DEFAULT 0 NOT NULL,
deleted_at INTEGER
);