use tokio_postgres::{Error, GenericClient, Row};

#[derive(Debug, serde::Serialize)]
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
}
