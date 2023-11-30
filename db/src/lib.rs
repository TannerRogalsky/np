pub type DB = sqlx::Postgres;
pub type Error = sqlx::Error;
pub type Result<T> = std::result::Result<T, Error>;
pub type Pool = sqlx::Pool<DB>;

pub async fn init<T>(db_url: T) -> Result<Pool>
where
    T: AsRef<str>,
{
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url.as_ref())
        .await?;

    sqlx::migrate!("src/migrations").run(&pool).await?;

    Ok(pool)
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Game {
    pub id: i32,
    pub game_id: i64,
}

impl Game {
    pub async fn fetch_or_insert(game_id: i64, pool: &Pool) -> Result<Self> {
        let maybe_game = sqlx::query_as::<_, Game>(
            "INSERT INTO games (game_id) VALUES ($1) ON CONFLICT DO NOTHING RETURNING *",
        )
        .bind(game_id)
        .fetch_optional(pool)
        .await?;
        match maybe_game {
            Some(game) => Ok(game),
            None => {
                sqlx::query_as::<_, Game>("SELECT * from games where game_id = $1")
                    .bind(game_id)
                    .fetch_one(pool)
                    .await
            }
        }
    }

    pub async fn players(&self, pool: &Pool) -> Result<Vec<Player>> {
        sqlx::query_as::<_, Player>("select * from players where game_id = $1")
            .bind(self.id)
            .fetch_all(pool)
            .await
    }

    pub async fn most_recent_tick(&self, pool: &Pool) -> Result<i64> {
        sqlx::query_as::<_, (i64,)>("select max((u.payload -> 'tick')::bigint) from universes u join players p on p.id = u.player_id where p.game_id = $1;")
            .bind(self.id)
            .fetch_one(pool)
            .await.map(|result| result.0)
    }

    pub async fn get_all_ticks(&self, pool: &Pool) -> Result<Vec<i64>> {
        sqlx::query_as::<_, (i64,)>("select distinct((u.payload -> 'tick')::bigint) from universes u join players p on p.id = u.player_id where p.game_id = $1 order by u.id;")
            .bind(self.id)
            .fetch_all(pool)
            .await.map(|result| result.into_iter().map(|e| e.0).collect())
    }

    pub async fn most_recent_universes_for_tick(
        &self,
        tick: i64,
        pool: &Pool,
    ) -> Result<Vec<Universe>> {
        const QUERY: &str = r#"select
        u.id,
        u.player_id,
        u.payload
    from
        (
        select
            u.*,
            row_number() over tick as row_num
        from
            universes u
        join players p on
            p.id = u.player_id
        where
            p.game_id = $1
    window
        tick as (partition by u.payload -> 'tick',
            u.player_id
        order by
            u.payload -> 'now' desc)) as u
    where
        row_num = 1
        and (u.payload -> 'tick')::bigint = $2;"#;
        sqlx::query_as::<_, Universe>(QUERY)
            .bind(self.id)
            .bind(tick)
            .fetch_all(pool)
            .await
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Player {
    pub id: i32,
    pub player_id: i64,
    pub game_id: i32,
    pub api_token: String,
}

impl Player {
    pub async fn save_report<'q, T>(&self, report: T, pool: &'q Pool) -> Result<i32>
    where
        T: Send + serde::Serialize,
    {
        sqlx::query_as::<_, (i32,)>(
            "insert into universes (player_id, payload) values ($1, $2) returning id",
        )
        .bind(self.id)
        .bind(sqlx::types::Json(report))
        .fetch_one(pool)
        .await
        .map(|r| r.0)
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Universe {
    pub id: i32,
    pub player_id: i32,
    pub payload: sqlx::types::Json<api::order::Report>,
}
