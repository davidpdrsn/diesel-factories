CREATE TABLE visited_cities
(
    user_id int not null,
    city_id int not null,
    primary key (user_id, city_id)
);
