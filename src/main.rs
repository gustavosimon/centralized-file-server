use axum::{extract::Path, http::StatusCode, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use std::{fs::{self, File}, io::Write};

#[derive(Serialize)]
struct FileEntity {
    id: i32,
    name: String,
}

/// Implementação da struct Files
impl FileEntity {
    pub fn new(id: i32, name: String) -> Self {
        Self {
            id,
            name
        }
    }
}

#[derive(Serialize, Deserialize)]
struct RawFile {
    name: String,
    content: Vec<u8>
}

impl RawFile {
    pub fn new(name: String, content: Vec<u8>) -> Self {
        Self {
            name,
            content
        }
    }
}

/// Classe principal do servidor de arquivos com armazenamento centralizado 
/// 
/// Os arquivos ficam armazenados na pasta `server`.
/// 
#[tokio::main]
async fn main() {
    //
    // Constrói as rotas do servidor HTTP
    // 
    let routes = Router::new().route("/list", get(list_files))
                                      .route("/upload", post(upload_file))
                                      .route("/download/:id", get(download_file));
    //
    // Levanta o servidor HTTP para ouvir as requisições
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, routes).await.unwrap();
}

/// Função responsável por listar os arquivos disponíveis no servidor
async fn list_files() -> (StatusCode, Json<Option<Vec<FileEntity>>>) {
    let result = get_files();
    if result.is_empty() {
        return (StatusCode::NO_CONTENT, Json(None));
    };
    (StatusCode::OK, Json(Some(result)))
}

/// Função responsável por tratar o upload de um arquivo do servidor
async fn upload_file(Json(raw_file): Json<RawFile>) -> (StatusCode, Json<Option<FileEntity>>) {
    let mut arquivo = File::create(format!("server/{}", raw_file.name)).unwrap();
    let result = match arquivo.write_all(&raw_file.content) {
        Ok(_) => (StatusCode::CREATED, Json(None)),
        Err(_) => (StatusCode::NOT_FOUND, Json(None)),
    };
    result
}

/// Função responsável por tratar o download de um arquivo do servidor
async fn download_file(Path(id): Path<i32>) -> (StatusCode, Json<Option<RawFile>>) {
    let files = get_files();
    for file in files {
        if file.id == id {
            let name = "server/".to_owned() + &file.name;
            let content = fs::read(name).unwrap();
            let raw_file = RawFile::new(file.name, content);
            return (StatusCode::OK, Json(Some(raw_file)));
        }
    }
    (StatusCode::NOT_FOUND, Json(None))
}

fn get_files() -> Vec<FileEntity> {
    let mut counter: i32 = 0;
    let mut files = Vec::new();
    for file in fs::read_dir("server").unwrap() {
        counter += 1;
        files.push(FileEntity::new(counter, file.unwrap().file_name().to_os_string().into_string().unwrap()));
    }
    files
}
