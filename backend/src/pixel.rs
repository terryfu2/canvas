use tokio_postgres::{Error, GenericClient, Row};
use serde_json;
use deadpool_postgres::Manager;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Pixel {
    pub x: i32,
    pub y: i32,
    pub colour: i32,
}

impl From<Row> for Pixel {
    fn from(row: Row) -> Self {
        Self {
            x: row.get(0),
            y: row.get(1),
            colour: row.get(2),
        }
    }
}

impl Pixel {
    pub async fn all<C: GenericClient>(client: &C) -> Result<Vec<Pixel>, Error> {
        let stmt = client.prepare("SELECT x, y, colour FROM canvas").await?;
        let rows = client.query(&stmt, &[]).await?;

        Ok(rows.into_iter().map(Pixel::from).collect())
    }

    pub async fn insert(client: deadpool::managed::Object<Manager>, data: String) -> Result<u64, Error> {
        println!("Here: {data:?}");
        let pixel: Pixel = serde_json::from_str(&data).unwrap();
        let stmt = client.prepare("INSERT INTO canvas (x,y,colour) VALUES ($1,$2,$3) ON CONFLICT (x,y) DO UPDATE SET x = $1, y = $2, colour = $3").await.unwrap();
        let result = client.execute(&stmt, &[&pixel.x, &pixel.y, &pixel.colour]).await.unwrap();

        Ok(result)
    }
}
