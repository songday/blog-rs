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


INSERT INTO "blog_tag" ("name") VALUES ('70年代');
INSERT INTO "blog_tag" ("name") VALUES ('80年代');
INSERT INTO "blog_tag" ("name") VALUES ('90年代');
INSERT INTO "blog_tag" ("name") VALUES ('复古怀旧');
INSERT INTO "blog_tag" ("name") VALUES ('国内影视');
INSERT INTO "blog_tag" ("name") VALUES ('国外影视');
INSERT INTO "blog_tag" ("name") VALUES ('国内音乐');
INSERT INTO "blog_tag" ("name") VALUES ('国外音乐');
INSERT INTO "blog_tag" ("name") VALUES ('像素、8-Bit');
INSERT INTO "blog_tag" ("name") VALUES ('电子游戏');
INSERT INTO "blog_tag" ("name") VALUES ('电子产品');
INSERT INTO "blog_tag" ("name") VALUES ('报刊杂志');
INSERT INTO "blog_tag" ("name") VALUES ('往期新闻');
INSERT INTO "blog_tag" ("name") VALUES ('生活用品');