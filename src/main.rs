use actix_web::{
    get, http::header::ContentType, middleware::Logger, web, App, HttpResponse, HttpServer,
    Responder, Result,
};
use mime;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use render::{render_markdown, write_markdown};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};
use linkify::LinkFinder;
mod render;

#[derive(Serialize)]
struct CommitEntry {
    post: CommitPost,
    path: String,
    patch: String,
}

#[derive(Deserialize, Serialize)]
struct CommitPost {
    username: String,
    message: String,
    document: String,
}

#[derive(Deserialize, PartialEq)]
enum CommitActionKind {
    Reject,
    Approve,
}
#[derive(Deserialize)]
struct CommitAction {
    path: String,
    kind: CommitActionKind,
}

fn authenticate(pass:&str)->bool{
    match std::env::var("pass"){
        Ok(v)=> &v==pass,
        Err(_)=> false
    }
}

fn run_commit(username: &str, message: &str, document: String)-> Result<String, Box<dyn std::error::Error>>{
    let contributors = fs::read_to_string("./Tetris-Community/contributors.txt")?;
    let contributors : Vec<String>= contributors.lines().map(|line| line.to_lowercase()).collect();
    let contributor_string = match contributors.contains(&username.to_lowercase()){
        true=>"0",
        false=>"1"
    };
    let mut child = Command::new("bash")
    .args(["src/commit.sh", username, message, contributor_string])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::inherit())
    .spawn()?;
    let stdin = child.stdin.as_mut().unwrap();
    stdin.write_all(document.as_bytes()).unwrap();
    let output = child.wait_with_output()?;
    let output = String::from_utf8_lossy(&output.stdout).to_string();
    let finder = LinkFinder::new();
    let links: Vec<_> = finder.links(&output).collect();
    if let Some(link) = links.iter().find(|l|{l.as_str().contains("pull/new")}){
        Ok(link.as_str().to_string())
    }else{
        log::error!("GIT COMMIT ERROR\n{output}");
        Err("unable to find PR".into())
    }
}

async fn commit_action(info: web::Json<CommitAction>, auth: BearerAuth) -> HttpResponse {
    if !authenticate(auth.token()){
        return HttpResponse::Forbidden().content_type(ContentType::plaintext()).body("not an admin");
    }
    match fs::read_to_string(&info.path) {
        Ok(s) => {
            let json: CommitPost = serde_json::from_str(&s).unwrap();
            if info.kind == CommitActionKind::Approve {

                match run_commit(&json.username, &json.message, json.document){
                    Err(e)=>{
                        log::error!("{}", e);
                        HttpResponse::ExpectationFailed()
                            .content_type(ContentType::plaintext())
                            .body("document unable to be committed, try again later")
                    },
                    Ok(output)=>{
                        fs::remove_file(&info.path).unwrap();
                        HttpResponse::Ok()
                            .content_type(ContentType::plaintext())
                            .body(output)
                    }
                }
            }else{
                fs::remove_file(&info.path).unwrap();
                HttpResponse::Ok()
                    .content_type(ContentType::plaintext())
                    .body("submitted!")
            }
        }
        Err(_) => HttpResponse::BadGateway()
            .content_type(ContentType::plaintext())
            .body("path not found"),
    }
}

async fn render_commit(info: String) -> Result<String> {
    let (_html, json) = render_markdown(&info)?;
    Ok(json)
}

async fn submit_commit(info: web::Json<CommitPost>) -> HttpResponse {
    if info.username.is_empty() {
        return HttpResponse::ExpectationFailed()
            .content_type(ContentType::plaintext())
            .body("empty username");
    }
    if info.message.is_empty() {
        return HttpResponse::ExpectationFailed()
            .content_type(ContentType::plaintext())
            .body("empty commit message");
    }
    if info.document.is_empty() {
        return HttpResponse::ExpectationFailed()
            .content_type(ContentType::plaintext())
            .body("empty document");
    }
    let document = info.document.replace("\n", "\r\n");
    let og = fs::read_to_string("./public/render/tetriscommunity.md").unwrap();
    if og == document {
        return HttpResponse::ExpectationFailed()
            .content_type(ContentType::plaintext())
            .body("no changes detected!");
    }
    if render_markdown(&document).is_err() {
        return HttpResponse::ExpectationFailed()
            .content_type(ContentType::plaintext())
            .body("document unable to render");
    }
    if let Err(e) = fs::write(
        format!(
            "commits/${:?}.md",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time travel happened somehow!!")
        ),
        serde_json::to_string(&info).expect("serde json translation"),
    ) {
        log::error!("{}", e);
        return HttpResponse::ExpectationFailed()
            .content_type(ContentType::plaintext())
            .body("document unable to be added, try again later");
    };
    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body("submitted!")
}


#[derive(Deserialize)]
struct PushEvent{
    #[serde(rename = "ref")] //ref is protected
    reference: String,
}

async fn webhook(info: web::Json<PushEvent>)->HttpResponse{
    if info.reference.ends_with("main"){
        log::warn!("Tetris-Community modified! Attempting to render new changes...");
        if let Err(e) = re_render() {
            log::error!("Markdown rendering error on file change: {:?}", e);
            HttpResponse::InternalServerError().content_type(ContentType::plaintext()).body("re-render failed")
        } else {
            log::info!("Tetris-Community re-render success!");
            HttpResponse::Ok().content_type(ContentType::plaintext()).body("re-render success")
        }
    }else{
        HttpResponse::BadRequest().content_type(ContentType::plaintext()).body("non-main branch")
    }
}

fn re_render() -> Result<(), Box<dyn std::error::Error>>{
    pull()?;
    write_markdown()?;
    Ok(())
}

fn pull() -> Result<(), Box<dyn std::error::Error>>{
    let mut child = Command::new("bash");
    child.arg("src/pull.sh");
    child.output()?;
    Ok(())
}

#[get("/")]
async fn index_html() -> HttpResponse {
    let s = fs::read_to_string("./public/index.html").unwrap();
    HttpResponse::Ok().content_type(ContentType::html()).body(s)
}

#[get("/{file_path}")]
async fn get_html(path: web::Path<String>) -> HttpResponse {
    let path = path.into_inner();
    let file_path = format!("./public/{path}.html");
    match fs::read(file_path) {
        Ok(something) => HttpResponse::Ok().content_type(mime::TEXT_HTML).body(something),
        Err(_) => HttpResponse::NotFound().body("Not found"),
    }
}

#[get("/js/{file_path}")]
async fn get_js(path: web::Path<String>) -> HttpResponse {
    let path = path.into_inner();
    let file_path = format!("./public/js/{path}");
    match fs::read(file_path) {
        Ok(something) => HttpResponse::Ok().content_type(mime::TEXT_JAVASCRIPT).body(something),
        Err(_) => HttpResponse::NotFound().body("Not found"),
    }
}

#[get("/css/{file_path}")]
async fn get_css(path: web::Path<String>) -> HttpResponse {
    let path = path.into_inner();
    let file_path = format!("./public/css/{path}");
    match fs::read(file_path) {
        Ok(something) => HttpResponse::Ok().content_type(mime::TEXT_CSS).body(something),
        Err(_) => HttpResponse::NotFound().body("Not found"),
    }
}


// HIGHLY HIGHLY RECOMEND REPLACING UNWARP LIKE JUST INCASE SEND LIKE A 404 or smthing
#[get("/raw")]
async fn raw() -> impl Responder {
    fs::read_to_string("./public/render/tetriscommunity.md").unwrap()
}

#[get("/render")]
async fn raw_render() -> HttpResponse {
    let s = fs::read_to_string("./public/render/tetriscommunity.html").unwrap();
    HttpResponse::Ok().content_type(ContentType::html()).body(s)
}

#[get("/data")]
async fn data() -> HttpResponse {
    let s = fs::read_to_string("./public/render/tetriscommunity.json").unwrap();
    HttpResponse::Ok().content_type("application/json").body(s)
}

#[get("/api/commits")]
async fn get_commits(pass:BearerAuth) -> HttpResponse {
    if !authenticate(pass.token()){
        return HttpResponse::Forbidden().content_type(ContentType::plaintext()).body("not an admin");
    }
    match read_commits(fs::read_to_string("./public/render/tetriscommunity.md").unwrap()) {
        Ok(json) => HttpResponse::Ok()
            .content_type("application/json")
            .body(json),
        Err(e) => {
            log::error!("commit read failed: {}", e);
            HttpResponse::ExpectationFailed()
                .content_type(ContentType::plaintext())
                .body("unable to read commits")
        }
    }
}

fn read_commits(original: String) -> std::result::Result<String, Box<dyn std::error::Error>> {
    let entries = fs::read_dir("commits").unwrap();
    let mut documents = Vec::new();
    for entry in entries {
        let path = entry?.path();
        let path_name = path.to_string_lossy().to_string();
        let entry: CommitPost = serde_json::from_str(&fs::read_to_string(&path)?)?;
        documents.push(CommitEntry {
            path: path_name,
            patch: diffy::create_patch(&original, &entry.document.replace("\n", "\r\n"))
                .to_string(),
            post: entry,
        });
    }
    Ok(serde_json::to_string(&documents)?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pull().unwrap();
    write_markdown().expect("Initial render of the markdown display page has failed: ");
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::new("%a %{User-Agent}i"))
            .route("/commit", web::post().to(render_commit))
            .route("/submit", web::post().to(submit_commit))
            .route("/manage", web::post().to(commit_action))
            .route("/webhook", web::post().to(webhook))
            .service(raw)
            .service(raw_render)
            .service(index_html)
            .service(get_commits)
            .service(data)
            .service(get_css)
            .service(get_js)
            .service(get_html)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
