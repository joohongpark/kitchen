//! 사진 업로드: 디코드 → 긴 변 기준 리사이즈 → JPEG 저장 → 메타데이터 기록

use std::io::Cursor;
use std::path::PathBuf;

use common::id::new_id;
use common::{Db, Error, Queryer, Result, time};
use image::ImageReader;
use image::codecs::jpeg::JpegEncoder;
use serde::Serialize;

const MAX_EDGE: u32 = 1600;
const JPEG_QUALITY: u8 = 82;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Photo {
    pub id: String,
    pub file_name: String,
    pub mime: String,
    pub width: i64,
    pub height: i64,
    pub size_bytes: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct PhotosRepository;

impl PhotosRepository {
    pub async fn insert<'c>(&self, db: impl Queryer<'c>, p: &Photo) -> Result<()> {
        sqlx::query(
            "INSERT INTO photos (id, file_name, mime, width, height, size_bytes, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&p.id)
        .bind(&p.file_name)
        .bind(&p.mime)
        .bind(p.width)
        .bind(p.height)
        .bind(p.size_bytes)
        .bind(&p.created_at)
        .execute(db)
        .await?;
        Ok(())
    }

    pub async fn find<'c>(&self, db: impl Queryer<'c>, id: &str) -> Result<Photo> {
        sqlx::query_as("SELECT * FROM photos WHERE id = ?")
            .bind(id)
            .fetch_optional(db)
            .await?
            .ok_or(Error::NotFound("사진"))
    }
}

#[derive(Debug, Clone)]
pub struct MediaService {
    db: Db,
    repo: PhotosRepository,
    dir: PathBuf,
}

impl MediaService {
    pub async fn new(db: Db, dir: PathBuf) -> Result<Self> {
        tokio::fs::create_dir_all(&dir).await.map_err(Error::internal)?;
        Ok(Self { db, repo: PhotosRepository, dir })
    }

    pub async fn upload(&self, bytes: Vec<u8>) -> Result<Photo> {
        // 디코딩/인코딩은 CPU 작업이라 블로킹 스레드에서
        let (jpeg, width, height) =
            tokio::task::spawn_blocking(move || process(&bytes)).await.map_err(Error::internal)??;

        let id = new_id();
        let file_name = format!("{id}.jpg");
        tokio::fs::write(self.dir.join(&file_name), &jpeg).await.map_err(Error::internal)?;

        let photo = Photo {
            id,
            file_name,
            mime: "image/jpeg".into(),
            width: width as i64,
            height: height as i64,
            size_bytes: jpeg.len() as i64,
            created_at: time::now(),
        };
        self.repo.insert(&self.db, &photo).await?;
        Ok(photo)
    }

    /// (메타데이터, 파일 내용)
    pub async fn read(&self, id: &str) -> Result<(Photo, Vec<u8>)> {
        let photo = self.repo.find(&self.db, id).await?;
        let bytes = tokio::fs::read(self.dir.join(&photo.file_name)).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound { Error::NotFound("사진 파일") } else { Error::internal(e) }
        })?;
        Ok((photo, bytes))
    }
}

fn process(bytes: &[u8]) -> Result<(Vec<u8>, u32, u32)> {
    let img = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(Error::internal)?
        .decode()
        .map_err(|e| Error::invalid(format!("이미지를 읽을 수 없습니다: {e}")))?;
    let img = if img.width().max(img.height()) > MAX_EDGE { img.thumbnail(MAX_EDGE, MAX_EDGE) } else { img };
    let rgb = img.to_rgb8();
    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, JPEG_QUALITY).encode_image(&rgb).map_err(Error::internal)?;
    Ok((out, rgb.width(), rgb.height()))
}
