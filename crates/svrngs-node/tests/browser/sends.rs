//! A carried app's send, in a browser (row 13, d05674c0; svrn-docs
//! `rings/current/CONTRACTS.md` §6 invariant 6, §7 "The yes"): the home
//! page's tick under a lend an app may send to is the person's yes to that
//! send, and a carried Taste sends its films through its own origin and
//! draws what comes back. The service is a stand-in
//! (tests/common/keeping.rs), so this runs in the gate with no model;
//! tests/carried.rs `a_carried_taste_sends` is the same send to a real
//! llama-server, by hand.

use super::*;
use keeping::{keeping, keeping_by, Kept};

/// Sam (`signing(2)`) lends each of `lends`, a name, the port it listens
/// on and its kind, to house, a lend of kind llama-server with the row
/// llama-server's terms ship, `POST /v1/embeddings`, as his yes keeps it.
fn lent_by_sam(dir: &Path, lends: &[(&str, u16, &str)]) {
    let declared = svrngs_node::lend::lends_dir(dir);
    std::fs::create_dir_all(&declared).unwrap();
    for (name, port, kind) in lends {
        let sends = match *kind {
            "llama-server" => json!(["POST /v1/embeddings"]),
            _ => json!([]),
        };
        let body = json!({ "groups": [NS], "port": port, "kind": kind, "limit": 3, "refused": [],
            "sends": sends });
        std::fs::write(declared.join(format!("{name}.json")), body.to_string()).unwrap();
    }
}

/// Where `at`'s browser opens `app`, once its open on the home page names
/// Sam's lend `label` as one it may send to, within 30 s: a lend another
/// computer's is listed only once that computer is heard from.
async fn opened_sending(at: &Node, app: &str, label: &str) -> String {
    let origin = carried::carried_at(at, app).await;
    let url = format!("http://{}/v1/lends?namespace={NS}", at.api_addr());
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    for _ in 0..60 {
        let lends: Value = (http.get(&url).bearer_auth(at.token()).send().await)
            .unwrap()
            .json()
            .await
            .unwrap();
        let listed = lends["lent"].as_array().into_iter().flatten();
        let open = listed.into_iter().find(|l| l["kind"] == app);
        let reading = open.and_then(|l| l["reading"].as_array().cloned());
        if (reading.into_iter().flatten()).any(|r| r["label"] == label && r["sends"].is_array()) {
            return origin;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    panic!("{app}'s open never names {label} to send to");
}

/// Each request `kept` holds by its first line, the method and path.
fn firsts(kept: &Kept) -> Vec<String> {
    let kept = kept.lock().unwrap();
    let lines = kept.iter().filter_map(|k| k.lines().next());
    lines
        .map(|l| l.rsplit_once(' ').map_or(l, |(at, _)| at).to_string())
        .collect()
}

in_both_engines!(
    /// **The send's tick is the yes to a send** (CONTRACTS §6.6, §7): Sam
    /// lends llama-server, a stand-in answering `POST /v1/embeddings`, its
    /// lend's `sends:` row that path; Probe, carried in house on ada's
    /// computer, names llama-server in its sends.txt. Opened by its typed
    /// address, its POST there is refused `unasked`, the service never
    /// asked. On ada's home page the open's fold draws, under Sam's
    /// llama-server, a tick of its own, unticked, saying "Sends to Sam’s
    /// llama-server at /v1/embeddings. Sam can see what it sends.", and
    /// the Open would post no send; ticked, it posts the lend's label; with
    /// the read unticked the send's tick is disabled and nothing is sent;
    /// the read ticked again and Open pressed, the yes, the new tab's POST
    /// reaches the service and its answer comes back; and on the home page
    /// again the send is ticked, by that yes alone. Planted, each watched
    /// failing: reach.js `Reach` posting no `sends`, and `Opening` drawing
    /// no send line.
    the_send_tick_is_the_yes_to_a_send
);
async fn the_send_tick_is_the_yes_to_a_send(engine: Engine) {
    let relay = relay().await;
    let dirs = [(); 2].map(|()| tempfile::tempdir().unwrap());
    let ada = node(dirs[0].path(), &relay, &signing(9)).await;
    let sam = joined(dirs[1].path(), &relay, &ada, 2, "Sam").await;
    let (port, asked) = keeping().await;
    lent_by_sam(dirs[1].path(), &[("llama-server", port, "llama-server")]);
    let folder = dirs[0].path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder(&ada, &folder).await;
    let page = "<!doctype html><title>Probe</title><body>Probe";
    let files = [
        ("index.html", page.into()),
        ("sends.txt", "llama-server\n".into()),
    ];
    carried::carry(&folder, "Probe", &files);
    let llama = label(2, "llama-server");
    let origin = opened_sending(&ada, "Probe", &llama).await;
    let api = ada.api_addr().port();
    let seen = drive(
        engine,
        "sends.mjs",
        json!({ "url": origin, "home": format!("http://svrngs.localhost:{api}/"), "words": "Open Probe",
            "label": llama, "path": "/v1/embeddings" }),
        None,
    )
    .await;
    eprintln!("the send's tick: {seen:#}");
    assert_eq!(
        (&seen["before"]["status"], &seen["before"]["refused"]),
        (&json!(403), &json!("unasked")),
        "before the tick, its send waits for the person's yes:\n{seen:#}"
    );
    let line = " Sends to Sam’s llama-server at /v1/embeddings. Sam can see what it sends.";
    assert_eq!(
        seen["open"],
        json!({ "line": line, "checked": [true, false], "disabled": [false, false],
            "reads": llama, "sends": "" }),
        "the open draws the send's tick under its read's, unticked, posting no send:\n{seen:#}"
    );
    assert_eq!(seen["error"], Value::Null, "{seen:#}");
    assert_eq!(
        seen["ticked"]["sends"], llama,
        "ticked, the Open posts the send:\n{seen:#}"
    );
    assert_eq!(
        (&seen["unread"]["disabled"], &seen["unread"]["sends"]),
        (&json!([false, true]), &json!("")),
        "with its read unticked a send is neither tickable nor posted:\n{seen:#}"
    );
    assert_eq!(
        (&seen["yes"]["status"], &seen["yes"]["refused"], &seen["yes"]["body"]),
        (
            &json!(200),
            &Value::Null,
            &json!(r#"{"data":[{"embedding":[0.5,0.25]}]}"#)
        ),
        "after the tick and the Open, the send reaches the service and its answer comes back:\n{seen:#}"
    );
    assert_eq!(
        (&seen["again"]["checked"], &seen["again"]["sends"]),
        (&json!([true, true]), &json!(llama)),
        "the last yes ticks the send:\n{seen:#}"
    );
    let kept = asked.lock().unwrap().clone();
    assert_eq!(
        firsts(&asked),
        ["POST /v1/embeddings"],
        "the service is asked once, after the yes: {kept:#?}"
    );
    assert!(
        kept[0].ends_with(r#"{"input":["a film"]}"#),
        "the body reaches it whole: {kept:#?}"
    );
    drop((ada, sam));
}

/// Three films, as Jellyfin's `/Items` lists a library's: two science
/// fiction, one not.
const FILMS: &str = r#"{"Items":[
{"Id":"1","Name":"Alien","ProductionYear":1979,"Genres":["Science Fiction","Horror"],"Overview":"A crew answers a distress call."},
{"Id":"2","Name":"Aliens","ProductionYear":1986,"Genres":["Science Fiction","Action"],"Overview":"Ripley goes back."},
{"Id":"3","Name":"Amélie","ProductionYear":2001,"Genres":["Comedy","Romance"],"Overview":"A waitress helps the people around her."}]}"#;

/// A stand-in Jellyfin: [`FILMS`] for every request.
fn jellyfin(_: &str) -> (&'static str, String) {
    ("application/json", FILMS.into())
}

/// A stand-in llama-server's `/v1/embeddings`: each text of the request's
/// `input` its fixed numbers, in order, science fiction one way and
/// anything else the other, as an OpenAI-shaped answer.
fn embeddings(request: &str) -> (&'static str, String) {
    let body = request.split_once("\r\n\r\n").map_or("", |(_, b)| b);
    let asked: Value = serde_json::from_str(body).unwrap_or_default();
    let texts = asked["input"].as_array().cloned().unwrap_or_default();
    let data: Vec<Value> = (texts.iter().enumerate())
        .map(|(index, text)| {
            let fiction = text.as_str().unwrap_or("").contains("Science Fiction");
            let embedding = if fiction { [1.0, 0.0] } else { [0.0, 1.0] };
            json!({ "index": index, "embedding": embedding })
        })
        .collect();
    ("application/json", json!({ "data": data }).to_string())
}

in_both_engines!(
    /// **A carried Taste sends its films through its own origin and draws
    /// what comes back** (EXTENSION_POINT §10 row 13; apps/taste/carried):
    /// Sam lends a stand-in Jellyfin of three films and a stand-in
    /// llama-server, its `sends:` row `POST /v1/embeddings`. Ada's computer
    /// carries apps/taste/carried as Taste, unchanged; on her home page she
    /// ticks its send under Sam's llama-server and opens it. Its page reads
    /// the films at its own origin, sends them, described, in one POST to
    /// the llama-server's `/v1/embeddings` there, and lists them; she rates
    /// Alien "Loved it" and it guesses Aliens, from the numbers that came
    /// back. The stand-in was asked that one POST, its `input` the three
    /// films in Jellyfin's order. Planted, watched failing: Taste's page
    /// posting to `v1/embedding`, a path its `sends:` row does not name.
    a_carried_taste_sends_and_draws_what_comes_back
);
async fn a_carried_taste_sends_and_draws_what_comes_back(engine: Engine) {
    let relay = relay().await;
    let dirs = [(); 2].map(|()| tempfile::tempdir().unwrap());
    let ada = node(dirs[0].path(), &relay, &signing(9)).await;
    let sam = joined(dirs[1].path(), &relay, &ada, 2, "Sam").await;
    let (films, on_films) = keeping_by(jellyfin).await;
    let (model, on_model) = keeping_by(embeddings).await;
    lent_by_sam(
        dirs[1].path(),
        &[
            ("jellyfin", films, "jellyfin"),
            ("llama-server", model, "llama-server"),
        ],
    );
    let folder = dirs[0].path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder(&ada, &folder).await;
    let taste = repo().join("apps/taste/carried");
    let files: Vec<(&str, String)> = ["index.html", "index.js", "reads.txt", "sends.txt"]
        .into_iter()
        .map(|f| (f, std::fs::read_to_string(taste.join(f)).unwrap()))
        .collect();
    carried::carry(&folder, "Taste", &files);
    let llama = label(2, "llama-server");
    opened_sending(&ada, "Taste", &llama).await;
    let api = ada.api_addr().port();
    let seen = drive(
        engine,
        "taste-sends.mjs",
        json!({ "home": format!("http://svrngs.localhost:{api}/"), "words": "Open Taste",
            "rate": "Alien", "as": "Loved it" }),
        None,
    )
    .await;
    eprintln!("Taste's run: {seen:#}");
    assert_eq!(
        seen["said"], "Sam’s llama-server described 3 films. Rate a few you’ve seen.",
        "its page sends the films and says what came back:\n{seen:#}"
    );
    assert_eq!(seen["error"], Value::Null, "{seen:#}");
    assert_eq!(
        seen["films"],
        json!(["Alien", "Aliens", "Amélie"]),
        "it lists the films it read:\n{seen:#}"
    );
    assert_eq!(
        seen["likely"],
        json!(["Aliens"]),
        "it guesses from the numbers that came back:\n{seen:#}"
    );
    let model_asked = on_model.lock().unwrap().clone();
    assert_eq!(
        firsts(&on_model),
        ["POST /v1/embeddings"],
        "Taste sends once, through its own origin, to its row's path: {model_asked:#?}"
    );
    let body = model_asked[0].split_once("\r\n\r\n").map_or("", |(_, b)| b);
    let sent: Value = serde_json::from_str(body).unwrap();
    let input: Vec<&str> = (sent["input"].as_array().unwrap().iter())
        .map(|t| t.as_str().unwrap())
        .collect();
    assert_eq!(input.len(), 3, "{sent:#}");
    for (text, film) in input
        .iter()
        .zip(["Alien (1979).", "Aliens (1986).", "Amélie (2001)."])
    {
        assert!(text.starts_with(film), "{film}: {sent:#}");
    }
    assert!(
        firsts(&on_films)
            .iter()
            .all(|l| l.starts_with("GET /Items?")),
        "Jellyfin is only read: {:?}",
        firsts(&on_films)
    );
    drop((ada, sam));
}
