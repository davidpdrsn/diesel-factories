CREATE TABLE countrys (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL
);
ALTER TABLE users
ADD COLUMN country_id integer NULL;