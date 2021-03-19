CREATE TABLE setting (
     id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
     blog_name TEXT NOT NULL,
     copyright TEXT(1024) NOT NULL,
     license TEXT(255) NOT NULL,
     manager_name TEXT(64) NOT NULL,
     manager_password TEXT(1024) NOT NULL
    , created_at INTEGER NOT NULL);


CREATE TABLE tag (
     id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
     name TEXT(16) NOT NULL,
     created_at INTEGER NOT NULL,
     updated_at INTEGER,
     is_deleted INTEGER DEFAULT 0 NOT NULL,
     deleted_at INTEGER
);

CREATE TABLE tag_usage (
   id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
   tag_id INTEGER NOT NULL,
   blog_id INTEGER NOT NULL,
   created_at INTEGER NOT NULL
);

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

----------------- old --------------------

CREATE TABLE "blog_tag" (
"name"  TEXT(16) NOT NULL,
CONSTRAINT "name" UNIQUE ("name" ASC)
);

CREATE TABLE "blog" (
"id"  INTEGER NOT NULL,
"title"  TEXT(64) NOT NULL,
"markdown_content"  TEXT(20480) NOT NULL,
"parsed_content"  TEXT(65535) NOT NULL,
"tags"  TEXT(256) NOT NULL,
"created_at"  INTEGER NOT NULL,
PRIMARY KEY ("id")
);
