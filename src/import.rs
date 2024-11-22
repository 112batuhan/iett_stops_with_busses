use std::collections::HashSet;

use iett_stops_with_busses::csv_parse::Route;
use iett_stops_with_busses::database::{
    delete_by_hatkodu, fetch_unique_hatkodus, get_db_connection, hatkodu_exist,
    insert_bus_route_stop,
};
use iett_stops_with_busses::request::{request_csv, request_soap};
use iett_stops_with_busses::xml_parse::{BusRouteMetadata, BusRouteStop, DurakDetay, HatServisi};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let client = reqwest::Client::new();
    let db_conn = get_db_connection().await.unwrap();

    let routes_url = "https://data.ibb.gov.tr/datastore/dump/46dbe388-c8c2-45c4-ac72-c06953de56a2";
    let routes: Vec<Route> = request_csv(client.clone(), routes_url).await.unwrap();
    let bus_route_names: HashSet<String> = routes
        .into_iter()
        .map(|route| route.route_short_name)
        .collect();

    // I didn't go for concurrent requests because it might cause rate limit or blacklist issues
    // We only need to do it once so it's fine to wait for few seconds for every buss
    println!("loaded {} bus routes", bus_route_names.len());
    for name in bus_route_names {
        if hatkodu_exist(&db_conn, &name).await.unwrap() {
            println!("{} exists, skipping", name);
            continue;
        }
        let bus_route_stops =
            request_soap::<DurakDetay, Vec<BusRouteStop>>(client.clone(), "DurakDetay_GYY", &name)
                .await
                .unwrap();
        println!("requested {} from api", name);
        for stop in bus_route_stops {
            insert_bus_route_stop(&db_conn, stop).await.unwrap();
        }
    }

    let inserted_bus_route_names = fetch_unique_hatkodus(&db_conn).await.unwrap();
    for name in inserted_bus_route_names {
        let bus_route_metadata = request_soap::<HatServisi, Option<BusRouteMetadata>>(
            client.clone(),
            "HatServisi_GYY",
            &name,
        )
        .await
        .unwrap();

        if bus_route_metadata.is_none() {
            println!("Deleting {} because inactive", name);
            delete_by_hatkodu(&db_conn, &name).await.unwrap()
        }
    }
}
