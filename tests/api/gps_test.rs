use openstreetmap_api::types;
use openstreetmap_api::Openstreetmap;
use pretty_assertions::assert_eq;
use rstest::*;
use wiremock::matchers::{method, path, query_param, QueryParamExactMatcher};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::utils::credentials;
use super::utils::no_credentials;

#[fixture]
fn gpx_response() -> &'static str {
    r#"
    <gpx version="1.0" creator="OpenStreetMap.org" xmlns="http://www.topografix.com/GPX/1/0">
        <trk>
            <name>20190626.gpx</name>
            <desc>Footpaths near Blackweir Pond, Epping Forest</desc>
            <url>https://api.openstreetmap.org/user/John%20Leeming/traces/3031013</url>
            <trkseg>
                <trkpt lat="51.6616100" lon="0.0534560">
                    <time>2019-06-26T14:27:58Z</time>
                </trkpt>
            </trkseg>
        </trk>
    </gpx>
    "#
}

#[fixture]
fn gpx_list() -> Vec<types::Track> {
    vec![types::Track {
        name: Some("20190626.gpx".into()),
        description: Some("Footpaths near Blackweir Pond, Epping Forest".into()),
        url: Some("https://api.openstreetmap.org/user/John%20Leeming/traces/3031013".into()),
        segments: vec![types::TrackSegment {
            points: vec![types::Waypoint {
                lat: 51.6616100,
                lon: 0.0534560,
                time: Some("2019-06-26T14:27:58Z".into()),
            }],
        }],
        ..Default::default()
    }]
}

#[rstest(bbox, page, request_params,
    case(
        types::BoundingBox {
            left: 1.0,
            bottom: 2.0,
            right: 3.0,
            top: 4.0,
        },
        None,
        vec![query_param("bbox", "1,2,3,4")],
    ),
    case(
        types::BoundingBox {
            left: 1.0,
            bottom: 2.0,
            right: 3.0,
            top: 4.0,
        },
        Some(1),
        vec![
            query_param("bbox", "1,2,3,4"),
            query_param("page", "1"),
        ],
    )
)]
#[actix_rt::test]
async fn test_get_by_bounding_box(
    no_credentials: types::Credentials,
    gpx_response: &str,
    bbox: types::BoundingBox,
    page: Option<u64>,
    request_params: Vec<QueryParamExactMatcher>,
    gpx_list: Vec<types::Track>,
) {
    /*
    GIVEN an OSM client
    WHEN calling the get_by_bounding_box() function
    THEN returns a list of GPX tracks
    */
    // GIVEN
    let mock_server = MockServer::start().await;
    let mut mock = Mock::given(method("GET"));

    for request_param in request_params {
        mock = mock.and(request_param);
    }

    mock.and(path("/api/0.6/trackpoints"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(gpx_response, "application/xml"))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), no_credentials);

    // WHEN
    let actual = client.gps().get_by_bounding_box(&bbox, page).await.unwrap();

    // THEN
    assert_eq!(actual, gpx_list);
}

#[rstest(gpx_id, case(10))]
#[actix_rt::test]
async fn test_delete(credentials: types::Credentials, gpx_id: u64) {
    /*
    GIVEN an OSM client
    WHEN calling the delete() function
    THEN returns nothing
    */

    // GIVEN
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path(format!("/api/0.6/gpx/{gpx_id}")))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let client = Openstreetmap::new(mock_server.uri(), credentials);

    // WHEN
    let actual = client.gps().delete(gpx_id).await.unwrap();

    // THEN
    assert_eq!(actual, ());
}
