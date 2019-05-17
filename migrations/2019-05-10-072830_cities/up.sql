CREATE TABLE cities (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL,
  country_id integer NOT NULL
);

ALTER TABLE users
    ADD COLUMN home_city_id integer,
    ADD COLUMN current_city_id integer;
