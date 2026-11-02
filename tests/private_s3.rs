use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use saras::images::image_storage::ImageStorage;
use saras::storage::storage::Storage;

fn handle(mut stream: TcpStream, objects: &mut HashMap<String, Vec<u8>>) {
    stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 4096];
    let header_end = loop {
        let n = stream.read(&mut chunk).unwrap();
        assert!(n > 0);
        bytes.extend_from_slice(&chunk[..n]);
        if let Some(index) = bytes.windows(4).position(|b| b == b"\r\n\r\n") {
            break index + 4;
        }
    };
    let headers = String::from_utf8(bytes[..header_end].to_vec()).unwrap();
    let mut request = headers.lines().next().unwrap().split_whitespace();
    let method = request.next().unwrap();
    let url = url::Url::parse(request.next().unwrap()).unwrap();
    let length = headers.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        key.eq_ignore_ascii_case("content-length").then(|| value.trim().parse::<usize>().unwrap())
    }).unwrap_or(0);
    while bytes.len() < header_end + length {
        let n = stream.read(&mut chunk).unwrap();
        assert!(n > 0);
        bytes.extend_from_slice(&chunk[..n]);
    }
    let authorization = headers.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        key.eq_ignore_ascii_case("authorization").then_some(value.trim())
    });
    let private = url.path() == "/private" || url.path().starts_with("/private/");
    let mut body = Vec::new();
    let status = if private && authorization.is_none() {
        "403 Forbidden"
    } else {
        let key = if private { "private-key" } else { "main-key" };
        assert!(authorization.unwrap().contains(&format!("Credential={key}/")));
        assert_eq!(url.host_str(), Some(if private { "private.invalid" } else { "main.invalid" }));
        match method {
            "PUT" => {
                objects.insert(url.path().to_string(), bytes[header_end..header_end + length].to_vec());
                "200 OK"
            }
            "GET" if url.query().is_some() => {
                assert!(private);
                body = b"<ListBucketResult><IsTruncated>false</IsTruncated><Contents><Key>orig/avatar/test.webp</Key></Contents></ListBucketResult>".to_vec();
                "200 OK"
            }
            "GET" | "HEAD" => {
                if let Some(data) = objects.get(url.path()) {
                    body = data.clone();
                    "200 OK"
                } else { "404 Not Found" }
            }
            "DELETE" => {
                objects.remove(url.path());
                "204 No Content"
            }
            _ => panic!("Неожиданный запрос: {method}"),
        }
    };
    write!(stream, "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
    if method != "HEAD" {
        stream.write_all(&body).unwrap();
    }
}

#[test]
fn private_s3_http_lifecycle_and_image_master() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let address = listener.local_addr().unwrap();
    let done = Arc::new(AtomicBool::new(false));
    let stop = done.clone();
    let server = std::thread::spawn(move || {
        let mut objects = HashMap::new();
        let mut count = 0;
        while !stop.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((stream, _)) => { handle(stream, &mut objects); count += 1; }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("{error}"),
            }
        }
        assert!(objects.is_empty());
        assert_eq!(count, 11);
    });
    let root = std::env::temp_dir().join(format!("saras-private-s3-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("saras.toml"), r#"
socket_path=""
is_dev=true
static_dir=""
main_image_size=""
main_image_format="webp"
image_formats=["webp"]
image_sizes=[{size="small",crop=false}]
vk_auth_url=""
ya_auth_url=""
ya_auth_client_id=""
ya_auth_client_secret=""
smtp_server=""
smtp_login=""
smtp_pwd=""
[legal_docs]
consent_key="test"
consent_version="test"
[selectel]
account_id="test"
proj_id="test"
proj_name="test"
container_name="legacy-main"
container_hostname=""
private_container_name="legacy-private"
svc_user_name="test"
svc_user_pwd="test"
token_lifetime_sec=3600
api_base_url="https://swift.invalid/v1"
map_api_base_url="https://swift.invalid/v1"
map_container_name="map"
map_container_hostname=""
[selectel.s3]
endpoint="http://main.invalid"
region="test"
bucket="main"
access_key_id="main-key"
secret_access_key="secret"
hostname=""
[selectel.private_s3]
endpoint="http://private.invalid"
region="test"
bucket="private"
access_key_id="private-key"
secret_access_key="secret"
hostname=""
"#).unwrap();
    let proxy = format!("http://{address}");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "private_s3_client", "--ignored", "--nocapture"])
        .current_dir(&root)
        .env("PRIVATE_S3_TEST", "1")
        .env("http_proxy", &proxy).env("HTTP_PROXY", &proxy)
        .env("https_proxy", &proxy).env("HTTPS_PROXY", &proxy)
        .env("no_proxy", "").env("NO_PROXY", "")
        .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let started = Instant::now();
    while child.try_wait().unwrap().is_none() {
        if started.elapsed() > Duration::from_secs(20) {
            child.kill().unwrap();
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    done.store(true, Ordering::SeqCst);
    server.join().unwrap();
    std::fs::remove_dir_all(&root).unwrap();
    assert!(output.status.success(), "{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
}

#[test]
#[ignore = "Запускается родительским тестом в отдельном процессе с искусственным конфигом"]
fn private_s3_client() {
    assert_eq!(std::env::var("PRIVATE_S3_TEST").unwrap(), "1");
    async_std::task::block_on(async {
        let private = Storage::with_private();
        let path = PathBuf::from("orig/avatar/test.webp");
        private.save_force_overwrite(b"master".to_vec(), &path).await.unwrap();
        assert!(private.exists(&path).await.unwrap());
        assert_eq!(private.open(&path).await.unwrap(), b"master");
        assert_eq!(private.list_prefix("orig").await.unwrap(), vec!["orig/avatar/test.webp"]);
        assert_eq!(ImageStorage::new().open_orig(Path::new("avatar/test")).await.unwrap(), b"master");
        let response = saras::isahc::get_async("http://private.invalid/private/orig/avatar/test.webp").await.unwrap();
        assert_eq!(response.status(), 403);
        ImageStorage::new().delete(&PathBuf::from("avatar/test")).await.unwrap();
        assert!(!private.exists(&path).await.unwrap());
        let main = Storage::new();
        main.save_force_overwrite(b"preview".to_vec(), &PathBuf::from("preview.webp")).await.unwrap();
        main.delete(PathBuf::from("preview.webp")).await.unwrap();
    });
}
