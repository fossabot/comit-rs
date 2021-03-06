use crate::{
    config::settings::AllowedOrigins,
    http_api,
    network::LocalPeerId,
    swap_protocols::{self, Facade, SwapId},
};
use warp::{self, filters::BoxedFilter, Filter, Reply};

pub const RFC003: &str = "rfc003";

pub fn swap_path(id: SwapId) -> String {
    format!("/{}/{}/{}", http_api::PATH, RFC003, id)
}

pub fn new_action_link(id: &SwapId, action: &str) -> String {
    format!("{}/{}", swap_path(*id), action)
}

pub fn create(
    dependencies: Facade,
    allowed_origins: &AllowedOrigins,
) -> BoxedFilter<(impl Reply,)> {
    let peer_id = dependencies.local_peer_id();
    let swaps = warp::path(http_api::PATH);
    let rfc003 = swaps.and(warp::path(RFC003));
    let peer_id = warp::any().map(move || peer_id.clone());
    let empty_json_body = warp::any().map(|| serde_json::json!({}));
    let dependencies = warp::any().map(move || dependencies.clone());

    let cors = warp::cors()
        .allow_methods(vec!["GET", "POST"])
        .allow_header("content-type");
    let cors = match allowed_origins {
        AllowedOrigins::None => cors.allow_origins(Vec::<&str>::new()),
        AllowedOrigins::All => cors.allow_any_origin(),
        AllowedOrigins::Some(hosts) => {
            cors.allow_origins::<Vec<&str>>(hosts.iter().map(|host| host.as_str()).collect())
        }
    };

    let preflight_cors_route = warp::options().map(warp::reply);

    let rfc003_post_swap = rfc003
        .and(warp::path::end())
        .and(warp::post())
        .and(dependencies.clone())
        .and(warp::body::json())
        .and_then(http_api::routes::rfc003::post_swap);

    let rfc003_get_swap = rfc003
        .and(warp::get())
        .and(dependencies.clone())
        .and(warp::path::param())
        .and(warp::path::end())
        .and_then(http_api::routes::rfc003::get_swap);

    let get_swaps = swaps
        .and(warp::get())
        .and(warp::path::end())
        .and(dependencies.clone())
        .and_then(http_api::routes::index::get_swaps);

    let rfc003_action = warp::method()
        .and(rfc003)
        .and(warp::path::param::<SwapId>())
        .and(warp::path::param::<
            swap_protocols::rfc003::actions::ActionKind,
        >())
        .and(warp::path::end())
        .and(warp::query::<http_api::action::ActionExecutionParameters>())
        .and(dependencies.clone())
        .and(warp::body::json().or(empty_json_body).unify())
        .and_then(http_api::routes::rfc003::action);

    let get_peers = warp::get()
        .and(warp::path("peers"))
        .and(warp::path::end())
        .and(dependencies.clone())
        .and_then(http_api::routes::peers::get_peers);

    let get_info_siren = warp::get()
        .and(warp::path::end())
        .and(warp::header::exact("accept", "application/vnd.siren+json"))
        .and(peer_id.clone())
        .and(dependencies.clone())
        .and_then(http_api::routes::index::get_info_siren);

    let get_info = warp::get()
        .and(warp::path::end())
        .and(peer_id)
        .and(dependencies)
        .and_then(http_api::routes::index::get_info);

    preflight_cors_route
        .or(rfc003_get_swap)
        .or(rfc003_post_swap)
        .or(rfc003_action)
        .or(get_swaps)
        .or(get_peers)
        .or(get_info_siren)
        .or(get_info)
        .recover(http_api::unpack_problem)
        .with(warp::log("http"))
        .with(cors)
        .boxed()
}
