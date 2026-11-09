//! **Ana's taste** (apps/taste/README.md): Sam lends his Jellyfin to House
//! and gives svrngs the account he made in it for apps that read it,
//! `svrngs share <its port> --readers`, the command's way of what the
//! Movie Night scene does on Sam's home page
//! (movie_night.rs). Bo lends a small model that turns a film's
//! description into numbers, llama-server with `--embeddings`, `svrngs
//! share <its port> --kind llama-server`. Ana joins House on her computer,
//! puts taste in her Downloads and pastes the README's one line, which
//! starts taste and lends it, `--kind taste --reads jellyfin --refuse
//! /diary`. She opens it from her home page, which is her yes to
//! what it reads, marks five of Sam's films, chooses her Letterboxd file,
//! and sees "You'll probably like" with the science fiction film she never
//! rated above the romantic comedy she never rated.
//!
//! Sam's library is the films check's scratch Jellyfin (films.rs
//! [`Jellyfin`]) holding [`FILMS`], each a copy of check.sh's test pattern
//! with an .nfo of its genres, plot and cast, beside the three test
//! patterns. Bo's model is [`GGUF`], read in front of it by a recorder
//! that writes down every byte it is sent. The scene is never-ran where
//! Jellyfin, llama-server or the GGUF is not on this computer.
//!
//! The invariant it relies on: **what Ana rated stays on her computer**:
//! her diary's own film is in no byte Sam's Jellyfin, Sam's or Bo's
//! svrngs, or Bo's model received; every list Bo's model is sent is the
//! whole of Sam's library in one order, whatever she rated
//! (apps/taste/model.mjs); only Ana ranks or rates; and no credential of
//! Sam's reaches her browser.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tokio::io::AsyncBufReadExt;
use tokio::process::{Child, Command};

use crate::films::{holding, node, Jellyfin};
use crate::harness::{browsers, repo, Engine};
use crate::people::{late, open_of, Person, Scene};

/// Bo's model, where the scene looks for it: `SCENES_EMBED_GGUF`, else
/// nomic-embed-text v1.5 at q8_0 (146 MB, Apache-2.0), as
/// huggingface.co/nomic-ai/nomic-embed-text-v1.5-GGUF gives it, in
/// `~/models/nomic-embed-text-v1.5-gguf/`.
const GGUF: &str = "models/nomic-embed-text-v1.5-gguf/nomic-embed-text-v1.5.Q8_0.gguf";

/// A film in Sam's library: its title, year, genres, plot and cast, and
/// what Ana says of it on taste's page, if anything.
struct Film {
    title: &'static str,
    year: u16,
    genres: &'static [&'static str],
    plot: &'static str,
    cast: &'static [&'static str],
    said: Option<&'static str>,
}

const SCI_FI: &str = "Starlight Freight";
const ROMCOM: &str = "Meet Cute in Manhattan";

/// Sam's films beside the test patterns: three science fiction films Ana
/// rates, two romantic comedies she rates, and one of each she never does.
const FILMS: &[Film] = &[
    Film { title: "Orbit of Glass", year: 2019, genres: &["Science Fiction"], plot: "A crew aboard a failing space station must reroute power from the reactor before their orbit decays into the gas giant below.", cast: &["Mara Quill", "Dev Osei"], said: Some("Loved it") },
    Film { title: "The Last Signal from Kepler", year: 2021, genres: &["Science Fiction", "Thriller"], plot: "An astronomer decodes a transmission from a distant exoplanet and learns the alien senders are already on their way to Earth.", cast: &["Ines Varga"], said: Some("Loved it") },
    Film { title: "Machine Tide", year: 2017, genres: &["Science Fiction", "Action"], plot: "In a future megacity, a rogue android hunts the engineer who built it, as robots rise against their makers.", cast: &["Tomas Reyl"], said: Some("Liked it") },
    Film { title: "Love at Second Brunch", year: 2018, genres: &["Romance", "Comedy"], plot: "Two rival food bloggers keep meeting at the same brunch spot and, after a fake dating scheme, fall in love.", cast: &["Lucy Hart", "Ben Ames"], said: Some("Not for me") },
    Film { title: "The Wedding Planner's Plus-One", year: 2020, genres: &["Romance", "Comedy"], plot: "A wedding planner who swore off romance hires a charming stranger as her date to her sister's wedding.", cast: &["Nora Bell"], said: Some("Not for me") },
    Film { title: SCI_FI, year: 2023, genres: &["Science Fiction", "Adventure"], plot: "A cargo pilot smuggling a stolen warp drive across the galaxy is chased by the spaceships of a ruthless empire.", cast: &["Kit Moreau"], said: None },
    Film { title: ROMCOM, year: 2022, genres: &["Romance", "Comedy"], plot: "A florist and a grumpy bookshop owner share a tiny Manhattan storefront and slowly fall for each other.", cast: &["Ellie Park", "Sam Rourke"], said: None },
];

/// Ana's own film: in her Letterboxd file, in no library, so only her diary
/// names it; and its Letterboxd address. Found in any byte that left her
/// computer, her diary left it.
const HERS: &str = "Paper Lanterns Over Quillwater";
const HERS_AT: &str = "https://boxd.it/qw7anaX";

/// The plant `SCENES_PLANT` names for this scene, if any: taste's own
/// (apps/taste/taste.mjs `PLANT`), set on the program Ana starts.
fn plant() -> Option<String> {
    let named = std::env::var("SCENES_PLANT").ok()?;
    named.strip_prefix("taste:").map(str::to_string)
}

/// Sam's library in `dir`: check.sh's three test patterns, `len` s each,
/// and each of [`FILMS`] a copy of the first with its .nfo, its data
/// locked so Jellyfin fetches nothing over it from the internet.
async fn library(dir: &Path) -> Result<(), String> {
    let script = repo().join("apps/jellyfin/check/check.sh");
    let made = Command::new(script)
        .args(["films", &dir.to_string_lossy(), "4"])
        .output()
        .await
        .unwrap();
    if made.status.code() == Some(4) {
        return Err(String::from_utf8_lossy(&made.stderr).trim().to_string());
    }
    assert!(
        made.status.success(),
        "check.sh made no films: {}",
        String::from_utf8_lossy(&made.stderr)
    );
    let pattern = dir.join("Test Pattern One (2026).mp4");
    let esc = |s: &str| s.replace('&', "&amp;").replace('<', "&lt;");
    for f in FILMS {
        let stem = format!("{} ({})", f.title, f.year);
        std::fs::copy(&pattern, dir.join(format!("{stem}.mp4"))).unwrap();
        let genres: String = f
            .genres
            .iter()
            .map(|g| format!("  <genre>{}</genre>\n", esc(g)))
            .collect();
        let cast: String = f
            .cast
            .iter()
            .map(|c| format!("  <actor><name>{}</name></actor>\n", esc(c)))
            .collect();
        let nfo = format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"yes\"?>\n<movie>\n  <title>{}</title>\n  <year>{}</year>\n  <plot>{}</plot>\n{genres}{cast}  <lockdata>true</lockdata>\n</movie>\n",
            esc(f.title), f.year, esc(f.plot)
        );
        std::fs::write(dir.join(format!("{stem}.nfo")), nfo).unwrap();
    }
    Ok(())
}

/// A recorder in front of a server on `to`: every request it is sent, its
/// method, path, headers and body, a JSON line each in the file named, then
/// passed on whole. It prints its port.
const RECORDER: &str = r#"
const [to, file] = process.argv.slice(1);
const http = require('node:http');
const fs = require('node:fs');
const s = http.createServer((q, r) => {
  const parts = []; q.on('data', (c) => parts.push(c)); q.on('end', () => {
    const body = Buffer.concat(parts);
    fs.appendFileSync(file, JSON.stringify({ method: q.method, url: q.url, headers: q.headers, body: body.toString() }) + '\n');
    const on = http.request({ host: '127.0.0.1', port: to, path: q.url, method: q.method, headers: { ...q.headers, host: `127.0.0.1:${to}` } }, (a) => { r.writeHead(a.statusCode, a.headers); a.pipe(r); });
    on.on('error', () => r.writeHead(502).end());
    on.end(body);
  });
});
s.listen(0, '127.0.0.1', () => console.log(s.address().port));
"#;

/// A port nothing holds now.
fn free_port() -> u16 {
    let free = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    free.local_addr().unwrap().port()
}

/// Bo's model: llama-server on `gguf` with `--embeddings`, up once it says
/// so, and its recorder writing to `heard`; the recorder's port, and both,
/// killed on drop. Or why it could not start where llama-server is not here.
async fn model(gguf: &Path, heard: &Path) -> Result<(u16, Child, Child), String> {
    let port = free_port();
    let server = Command::new("llama-server")
        .args([
            "-m",
            &gguf.to_string_lossy(),
            "--embeddings",
            "--host",
            "127.0.0.1",
        ])
        .args([
            "--port",
            &port.to_string(),
            "-c",
            "2048",
            "-b",
            "2048",
            "-ub",
            "2048",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn();
    let server =
        server.map_err(|e| format!("llama-server is not on this computer's PATH ({e})"))?;
    let health = format!("http://127.0.0.1:{port}/health");
    let mut up = false;
    for _ in 0..240 {
        if reqwest::get(&health)
            .await
            .is_ok_and(|r| r.status().is_success())
        {
            up = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    if !up {
        late(format!(
            "Bo's llama-server did not come up on {port} within 60 s"
        ))
        .await;
    }
    let mut recorder = Command::new("node")
        .args(["-e", RECORDER, &port.to_string(), &heard.to_string_lossy()])
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("node runs");
    let mut printed = tokio::io::BufReader::new(recorder.stdout.take().unwrap()).lines();
    let at = printed
        .next_line()
        .await
        .unwrap()
        .expect("it prints its port");
    Ok((at.parse().unwrap(), server, recorder))
}

/// The README's line, as Ana pastes it: the first indented line that runs
/// `node` (apps/taste/README.md, "Start it").
fn readme_line() -> String {
    let readme = std::fs::read_to_string(repo().join("apps/taste/README.md")).unwrap();
    let line = readme.lines().find(|l| l.starts_with("    node "));
    line.expect("the README gives a line to paste")
        .trim()
        .to_string()
}

/// Ana's taste: apps/taste copied to her Downloads, as she put it there,
/// and the README's `line` run in a shell in her home, with this build's
/// svrngs on her `PATH` and nothing else of this computer's environment, as
/// her terminal runs it. What it said, once it says taste is on; her
/// window, closed on drop.
/// `taste:lender-token` writes `lender`, Sam's own token, into her copy of
/// taste's page, which her browser is then sent.
async fn starts_taste(ana: &Person, line: &str, lender: &str) -> (Vec<String>, Window) {
    let downloads = ana.home.join("Downloads/taste");
    copy_dir(&repo().join("apps/taste"), &downloads);
    if plant().as_deref() == Some("lender-token") {
        let page = downloads.join("page/page.js");
        let js = std::fs::read_to_string(&page).unwrap();
        std::fs::write(&page, format!("{js}\nexport const TOKEN = '{lender}';\n")).unwrap();
    }
    let bin = Path::new(env!("CARGO_BIN_EXE_svrngs"))
        .parent()
        .unwrap()
        .to_path_buf();
    let path = std::env::join_paths(std::iter::once(bin).chain(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    )))
    .unwrap();
    let mut run = Command::new("sh");
    run.args(["-c", line])
        .env_clear()
        .env("HOME", &ana.home)
        .env("USER", "ana")
        .env("PATH", path)
        .env("NO_COLOR", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .kill_on_drop(true);
    if let Some(p) = plant() {
        run.env("TASTE_PLANT", p);
    }
    let mut taste = run.spawn().expect("sh runs");
    let mut said = tokio::io::BufReader::new(taste.stdout.take().unwrap()).lines();
    let mut lines = vec![];
    let began = Instant::now();
    while began.elapsed() < Duration::from_secs(90) {
        match tokio::time::timeout(Duration::from_secs(90), said.next_line()).await {
            Ok(Ok(Some(l))) => {
                let on = l.starts_with("Keep this window open");
                lines.push(l);
                if on {
                    return (lines, Window(taste));
                }
            }
            _ => break,
        }
    }
    let mut err = String::new();
    if let Some(mut e) = taste.stderr.take() {
        use tokio::io::AsyncReadExt;
        let _ = tokio::time::timeout(Duration::from_secs(2), e.read_to_string(&mut err)).await;
    }
    panic!("taste did not say it is on: {lines:#?} {err}");
}

/// Ana's terminal window with the line running in it. Closing it hangs up
/// on the line's process group, as a terminal does, not on `sh` alone:
/// Ubuntu's sh, dash, runs `node` as its child rather than exec it as the
/// Mac's does, so killing `sh` left taste holding its place, and pasting
/// again said another program was using it (the gate image, 2026-11-08).
/// The shell's own `kill`, not /bin/kill: Ubuntu's, procps-ng 4.0.4, given
/// `-HUP -<pid>` hung up on the test's own group too.
struct Window(Child);

impl Drop for Window {
    fn drop(&mut self) {
        if let Some(pid) = self.0.id() {
            let _ = std::process::Command::new("sh")
                .args(["-c", &format!("kill -HUP -{pid}")])
                .status();
        }
    }
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let (p, name) = (e.path(), e.file_name());
        if name == "__pycache__" {
            continue;
        }
        if p.is_dir() {
            copy_dir(&p, &to.join(name));
        } else {
            std::fs::copy(&p, to.join(name)).unwrap();
        }
    }
}

/// `method <path>` of `body` at the lend whose origin on `who`'s computer
/// is `origin`, as a program there asks it, with that computer's token
/// (pantry.mjs `probe`): its status, the door's refusal header, its body.
async fn as_program(
    who: &Person,
    origin: &str,
    method: &str,
    path: &str,
    body: &Value,
) -> (u16, Option<String>, String) {
    let token = std::fs::read_to_string(who.home.join(".svrngs/token")).unwrap();
    let host = origin.split('/').nth(2).unwrap().to_string();
    let port = host.rsplit(':').next().unwrap();
    let url = format!("http://127.0.0.1:{port}{path}");
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let sent = match method {
        "GET" => client.get(url),
        _ => client
            .post(url)
            .header("content-type", "application/json")
            .body(body.to_string()),
    };
    let got = sent
        .header("host", host)
        .bearer_auth(token.trim())
        .send()
        .await
        .unwrap();
    let status = got.status().as_u16();
    let refused = got
        .headers()
        .get("x-svrngs-refused")
        .map(|v| v.to_str().unwrap_or_default().to_string());
    (status, refused, got.text().await.unwrap_or_default())
}

/// Ana's browser, taste.mjs on `args`: the JSON line it printed.
async fn ana_sees(args: &Value) -> Value {
    let script = repo().join("crates/svrngs/tests/scenes/taste.mjs");
    let run = Command::new("node")
        .arg(script)
        .arg(args.to_string())
        .env("SVRNGS_BROWSERS", browsers())
        .kill_on_drop(true)
        .output();
    let out = tokio::time::timeout(Duration::from_secs(500), run);
    let out = out.await.expect("Ana's browser ends within 500 s").unwrap();
    let said = String::from_utf8_lossy(&out.stdout);
    let line = said.lines().last().unwrap_or_default();
    serde_json::from_str(line).unwrap_or_else(|_| {
        panic!(
            "taste.mjs printed no JSON: {said}{}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

/// Each `POST /v1/embeddings` the recorder at `heard` wrote down: the list
/// it sent, and every request it was sent at all.
fn embedded(heard: &Path) -> (Vec<Vec<String>>, Vec<Value>) {
    let text = std::fs::read_to_string(heard).unwrap_or_default();
    let all: Vec<Value> = text
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    let lists = all
        .iter()
        .filter(|r| r["method"] == "POST" && r["url"] == "/v1/embeddings")
        .map(|r| {
            let body: Value =
                serde_json::from_str(r["body"].as_str().unwrap_or_default()).unwrap_or(Value::Null);
            let input = body["input"].as_array().cloned().unwrap_or_default();
            input
                .iter()
                .map(|i| i.as_str().unwrap_or_default().to_string())
                .collect()
        })
        .collect();
    (lists, all)
}

/// Sam's films as his own account lists them, in the one order taste sends
/// a model (apps/taste/model.mjs `Catalog`): one library, so by id.
async fn in_order(jf: &Jellyfin, admin: &str) -> Vec<String> {
    let got = reqwest::Client::new()
        .get(format!(
            "{}/Items?Recursive=true&IncludeItemTypes=Movie",
            jf.at()
        ))
        .header("authorization", format!("MediaBrowser Token=\"{admin}\""))
        .send()
        .await
        .unwrap();
    let got: Value = got.json().await.unwrap();
    let mut items: Vec<(String, String)> = (got["Items"].as_array().into_iter().flatten())
        .map(|i| {
            (
                i["Id"].as_str().unwrap_or_default().to_string(),
                i["Name"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect();
    items.sort();
    items.into_iter().map(|(_, name)| name).collect()
}

scene!(
    /// Planted, watched failing, each by `SCENES_PLANT` (apps/taste/
    /// taste.mjs `PLANT`): `taste:constant`, a scorer that knows nothing;
    /// `taste:subset`, taste sends the model only the films her diary names;
    /// `taste:rated-first`, her rated films first in the list; `taste:bo`,
    /// the owner check off; `taste:lender-token`, Sam's own token in her
    /// copy of taste's page.
    taste
);
async fn taste(engine: Engine) {
    let began = Instant::now();
    let scene = Scene::new("taste", engine).await;
    let gguf = std::env::var_os("SCENES_EMBED_GGUF")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(GGUF));
    if !gguf.is_file() {
        return scene.never_ran(&format!(
            "Bo's model is not on this computer ({})",
            gguf.display()
        ));
    }
    let media = scene.scratch("films");
    std::fs::create_dir_all(&media).unwrap();
    let media = media.canonicalize().unwrap();
    if let Err(why) = library(&media).await {
        return scene.never_ran(&why);
    }
    // What Bo's model was sent, in a dir of its own, so `holding` reads it.
    let heard_in = scene.scratch("heard");
    std::fs::create_dir_all(&heard_in).unwrap();
    let heard = heard_in.join("requests.jsonl");
    let (model_port, _llama, _recorder) = match model(&gguf, &heard).await {
        Ok(m) => m,
        Err(why) => return scene.never_ran(&why),
    };
    let jf = match Jellyfin::up_with(scene.scratch("jellyfin"), Some(&media)).await {
        Ok(jf) => jf,
        Err(why) => return scene.never_ran(&why),
    };
    let admin = jf.read("secrets/admin_token");
    let dump = scene.scratch("received");

    // 1. Sam makes House, lends his Jellyfin, makes the account for apps
    // that read it in Jellyfin's own pages, and gives svrngs that account.
    let (sam, group) = scene.founds("Sam", "House").await;
    let port = jf.port.to_string();
    let jellyfin = ["share", &port, "--kind", "jellyfin", "--group", "House"];
    let shared = scene.command(&sam, &jellyfin, "\n").await;
    let how = shared
        .lines()
        .find(|l| l.contains("asks each person to sign in once"))
        .unwrap_or_else(|| panic!("share tells Sam nothing of accounts: {shared}"));
    let secrets = [(
        "SECRETS",
        jf.scratch.join("r/secrets").to_string_lossy().to_string(),
    )];
    let made = node("lender.mjs", &[how, &jf.at(), "sam", "readers"], &secrets).await;
    assert!(
        made.get("stopped").is_none(),
        "Sam makes the readers' account: {made:#}"
    );
    let password = jf.read("secrets/readers_pw");
    let readers = [&jellyfin[..], &["--readers"]].concat();
    scene
        .command(&sam, &readers, &format!("readers\n{password}\n"))
        .await;
    let kept = sam
        .home
        .join(".svrngs/secrets/lends/jellyfin/readers/authorization");
    let readers_token = std::fs::read_to_string(&kept).unwrap_or_default();
    let readers_token = readers_token
        .split('"')
        .nth(1)
        .unwrap_or_default()
        .to_string();

    // 2. Bo joins and lends his model.
    let link = sam.invites(&group).await;
    let bo = scene.joins("Bo", &link, "House").await;
    let model_port = model_port.to_string();
    let lends = [
        "share",
        &model_port,
        "--kind",
        "llama-server",
        "--group",
        "House",
    ];
    let shared = scene.command(&bo, &lends, "\n").await;
    assert!(shared.contains("You lend llama-server"), "{shared}");

    // 3. Ana joins on her computer, and starts taste with the README's line.
    let ana = scene.joins("Ana", &link, "House").await;
    ana.lent_by(&group, "Sam", "jellyfin").await;
    ana.lent_by(&group, "Bo", "llama-server").await;
    let line = readme_line();
    let (terminal, first) = starts_taste(&ana, &line, &admin).await;
    eprintln!("Ana pasted `{line}`; her terminal said: {terminal:#?}");
    // She closes the window, and pastes the line again another evening:
    // taste takes the place it took, and its lend still reaches it.
    drop(first);
    tokio::time::sleep(Duration::from_millis(500)).await;
    let (again, _taste) = starts_taste(&ana, &line, &admin).await;
    assert_eq!(again, terminal, "pasted again, taste says what it said");
    let opener = terminal
        .iter()
        .find_map(|l| l.split('“').nth(1)?.split('”').next())
        .unwrap_or_else(|| panic!("taste names what to press: {terminal:#?}"))
        .to_string();
    ana.lent_by(&group, "Ana", "taste").await;

    // 4. She opens it from her home page, marks five films, chooses her
    // Letterboxd file, and reads the page.
    let files = scene.scratch("ana-files");
    std::fs::create_dir_all(&files).unwrap();
    let letterboxd = files.join("ratings.csv");
    let csv = format!("Date,Name,Year,Letterboxd URI,Rating\n2026-11-01,{HERS},1987,{HERS_AT},4.5\n2026-11-02,Orbit of Glass,2019,https://boxd.it/orb1t,5\n");
    std::fs::write(&letterboxd, csv).unwrap();
    let home = open_of(&ana.home_page()).expect("svrngs opens it").1;
    let rates: Vec<_> = FILMS
        .iter()
        .filter_map(|f| Some((f.title, f.said?)))
        .collect();
    let args = json!({
        "engine": Engine::Chromium.as_str(), "home": home, "opener": opener,
        "rates": rates, "letterboxd": letterboxd, "watch": SCI_FI, "after": "Loved it",
        "dump": dump,
    });
    let seen = ana_sees(&args).await;
    eprintln!("what Ana saw: {seen:#}");
    if let Some(stopped) = seen.get("stopped") {
        let error = stopped["error"].as_str().unwrap_or_default();
        if error.contains("Timeout") || error.contains("exceeded") {
            late(format!(
                "Ana's steps did not go within their bounds: {seen:#}"
            ))
            .await;
        }
        panic!("Ana's steps did not go as the scene says: {seen:#}");
    }
    let lists = &seen["lists"];
    let titles = |part: &str| -> Vec<String> {
        (lists[part].as_array().into_iter().flatten())
            .filter_map(|t| t.as_str().map(str::to_string))
            .collect()
    };
    // The first time is an exercise: no guess until five films are rated
    // (apps/taste/page/page.js `FIRST`).
    let opened = &seen["opened"]["lists"];
    assert_eq!(
        (&opened["heading"], &opened["likely"], &opened["unlikely"]),
        (
            &json!("First, rate 5 films you’ve seen (0 of 5 so far)"),
            &json!([]),
            &json!([])
        ),
        "before she rates, taste asks for five and guesses nothing: {opened:#}"
    );
    let rated = titles("rated");
    for f in FILMS.iter().filter(|f| f.said.is_some()) {
        assert!(
            rated.iter().any(|t| t == f.title),
            "her rating of {} is kept: {lists:#}",
            f.title
        );
    }

    // Ratings stay home: her own film is in her diary, and in no byte that
    // Sam's Jellyfin, Sam's or Bo's svrngs, or Bo's model received.
    let diary = ana.home.join(".svrngs/taste/diary.json");
    let diary = std::fs::read_to_string(&diary).unwrap_or_default();
    assert!(
        diary.contains(HERS) && diary.contains(HERS_AT),
        "her diary keeps her Letterboxd film: {diary}"
    );
    let (lists_sent, heard_all) = embedded(&heard);
    for mark in [HERS, HERS_AT] {
        let mut held = holding(&jf.scratch, mark.as_bytes());
        held.extend(holding(&sam.home, mark.as_bytes()));
        held.extend(holding(&bo.home, mark.as_bytes()));
        held.extend(holding(&heard_in, mark.as_bytes()));
        assert!(
            held.is_empty(),
            "her diary's {mark} left her computer: {held:?}"
        );
    }

    // Bo's model never learns which films she rated: every list it was
    // sent is Sam's whole library, in the one order.
    let order = in_order(&jf, &admin).await;
    assert!(
        !lists_sent.is_empty(),
        "Bo's model was asked: {heard_all:#?}"
    );
    for (i, sent) in lists_sent.iter().enumerate() {
        let titles: Vec<&str> = (sent.iter())
            .map(|t| t.split(" (").next().unwrap_or(t).trim_end_matches('.'))
            .collect();
        assert_eq!(
            titles, order,
            "list {i} Bo's model was sent is Sam's whole library in its one order"
        );
    }
    eprintln!(
        "Bo's model was asked {} times, each the whole list of {}",
        lists_sent.len(),
        order.len()
    );

    // After watching: she follows the guess to Sam's Jellyfin, comes back,
    // is asked how it was once, and her answer is kept with her ratings.
    let host = |at: &str| at.split('/').nth(2).unwrap_or_default().to_string();
    let jf_at = ana.lent_by(&group, "Sam", "jellyfin").await;
    assert_eq!(
        (
            seen["watch"].as_str(),
            host(seen["watched"].as_str().unwrap_or_default())
        ),
        (Some("Watch from Sam’s library"), host(&jf_at)),
        "her guess opens at Sam's Jellyfin: {seen:#}"
    );
    assert_eq!(
        (
            seen["after"]
                .as_str()
                .map(|a| a.lines().next().unwrap_or_default()),
            &seen["asked_after"]
        ),
        (Some(format!("How was {SCI_FI}?").as_str()), &json!(false)),
        "back from watching, taste asks how it was, once: {seen:#}"
    );
    let rows: Value = serde_json::from_str(&diary).unwrap_or_default();
    let kept = (rows["rows"].as_array().into_iter().flatten())
        .find(|r| r["title"] == SCI_FI)
        .map(|r| r["rating"].clone());
    assert_eq!(
        kept,
        Some(json!(5)),
        "Loved it, after watching, is kept: {diary}"
    );
    eprintln!("back from Jellyfin: {}", seen["back"]);

    // Prediction: of the films she never rated, the science fiction one
    // first, before the romantic comedy.
    let unrated = [titles("likely"), titles("unlikely"), titles("unguessed")].concat();
    let at = |t: &str| unrated.iter().position(|u| u == t);
    assert!(
        matches!((at(SCI_FI), at(ROMCOM)), (Some(s), Some(r)) if s < r),
        "{SCI_FI} comes before {ROMCOM} in what she'll probably like: {lists:#}"
    );
    assert!(
        titles("unguessed").is_empty() && !titles("likely").is_empty(),
        "taste guesses every film she never rated: {lists:#}"
    );
    assert_eq!(
        titles("likely").first().map(String::as_str),
        Some(SCI_FI),
        "{lists:#}"
    );
    // Her page's words are hers to read: none she must look up.
    let text = seen["text"].as_str().unwrap_or_default().to_lowercase();
    for word in ["embedding", "vector", "cosine", "token", "port", "lend"] {
        let said = text
            .split(|c: char| !c.is_alphanumeric())
            .any(|w| w.starts_with(word));
        assert!(!said, "taste's page says “{word}”: {text}");
    }

    // Only Ana: Bo, through the group, is refused her ranking and rating;
    // the door refuses /diary to everyone, Ana's own page too.
    let at_bo = bo.lent_by(&group, "Ana", "taste").await;
    let film = json!({ "library": "x", "id": "1", "title": SCI_FI });
    let ranks = as_program(&bo, &at_bo, "POST", "/rank", &json!({ "films": [film] })).await;
    assert_eq!(ranks.0, 403, "Bo is refused Ana's ranking: {ranks:?}");
    assert!(
        ranks.2.contains("not-yours"),
        "taste refuses Bo, not svrngs: {ranks:?}"
    );
    let rates = as_program(
        &bo,
        &at_bo,
        "POST",
        "/rate",
        &json!({ "film": film, "said": "loved" }),
    )
    .await;
    assert_eq!(rates.0, 403, "Bo is refused rating as Ana: {rates:?}");
    let diary_bo = as_program(&bo, &at_bo, "GET", "/diary", &Value::Null).await;
    assert!(
        diary_bo.0 == 403 && diary_bo.1.is_some() && !diary_bo.2.contains(HERS),
        "the door refuses Bo /diary: {diary_bo:?}"
    );
    let diary_ana = &seen["diary"];
    assert!(
        diary_ana["status"] == 403 && !diary_ana["refused"].is_null(),
        "the door refuses /diary to her own page: {diary_ana:#}"
    );

    // No credential of Sam's reached Ana: in nothing her browser received,
    // nor in her svrngs's or taste's files.
    for (what, secret) in [
        ("Sam's own token", &admin),
        ("the readers' token", &readers_token),
        ("the readers' password", &password),
    ] {
        assert!(!secret.is_empty(), "{what} is known to the scene");
        let mut held = holding(&dump, secret.as_bytes());
        held.extend(holding(&ana.home, secret.as_bytes()));
        assert!(held.is_empty(), "{what} reached Ana: {held:?}");
    }
    eprintln!(
        "the taste scene took {:.0} s",
        began.elapsed().as_secs_f64()
    );
    drop((sam, bo, ana, jf));
}
