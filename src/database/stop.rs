use sqlx::PgPool;
use anyhow::{Context, Result};

use crate::models::bus::BusRouteStopResponse;

// pub async fn fetch_stop_coordinates(
//     pool: &PgPool,
//     hatkodu: &str,
//     direction: &str,
// ) -> Result<Vec<Coordinates>> {
//     let coordinates = sqlx::query_as!(
//         Coordinates,
//         r#"
//         SELECT
//             xkoordinati as x, ykoordinati as y
//         FROM
//             bus_route_stops
//         WHERE
//             hatkodu = $1 AND yon = $2
//         ORDER BY sirano ASC
//         "#,
//         hatkodu,
//         direction
//     )
//     .fetch_all(pool)
//     .await?;
//     Ok(coordinates)
// }

pub async fn fetch_hatkodu_by_durakkodu(pool: &PgPool, durakkodu: u32) -> Result<Vec<String>> {
    let hatkodus = sqlx::query!(
        r#"
            SELECT hatkodu
            FROM bus_route_stops
            WHERE durakkodu = $1
            "#,
        durakkodu as i32
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|record| record.hatkodu)
    .collect();

    Ok(hatkodus)
}

pub async fn fetch_stop_info_by_durakkodu(
    pool: &PgPool,
    durakkodu: u32,
) -> Result<BusRouteStopResponse> {
    let stop_info = sqlx::query_as!(
        BusRouteStopResponse,
        r#"
        SELECT 
            durakkodu,
            durakadi,
            xkoordinati,
            ykoordinati,
            duraktipi,
            isletmebolge,
            isletmealtbolge,
            ilceadi
        FROM bus_route_stops
        WHERE durakkodu = $1
        LIMIT 1
        "#,
        durakkodu as i32
    )
    .fetch_optional(pool)
    .await?;

    stop_info.context("Missing bus stop in Database")
}
