CREATE TABLE IF NOT EXISTS games (
    id SERIAL primary key,
    game_id bigint unique
);

CREATE TABLE IF NOT EXISTS players (
    id SERIAL primary key,
    player_id bigint,
    game_id int references games(id) on delete cascade,
    api_token varchar(128),
    unique(player_id, game_id)
);

CREATE TABLE IF NOT EXISTS ticks (
    id SERIAL primary key,
    game_id int references games(id) on delete cascade,
    tick_id int,
    UNIQUE (game_id, tick_id)
);

CREATE TABLE IF NOT EXISTS universes (
    id SERIAL primary key,
    tick_id int references ticks(id) on delete cascade,
    player_id int references players(id) on delete cascade,
    payload JSONB,
    UNIQUE (tick_id, player_id)
);