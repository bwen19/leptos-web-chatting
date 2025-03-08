cfg_if::cfg_if! { if #[cfg(feature = "ssr")] {
    use std::{io, ffi::OsStr, path::{Path, PathBuf}, time::SystemTime, sync::Arc};
    use axum::{body::Bytes, BoxError};
    use tokio::{fs::File, io::BufWriter};
    use tokio_util::io::StreamReader;
    use futures_util::{Stream, TryStreamExt};
    use multer::Field;
    use uuid::Uuid;
    use qrcode::QrCode;
    use image::Luma;
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use crate::{DateTime, Result, Error, Config, Store};
}}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct FileMeta {
    pub name: String,
    pub url: String,
    pub img: bool,
}

// ==================== // FileManager // ==================== //

#[cfg(feature = "ssr")]
pub struct FileManager;

#[cfg(feature = "ssr")]
impl FileManager {
    /// Save the uploaded avatar
    ///
    pub async fn save_avatar(
        user_id: i64,
        field: Field<'static>,
        config: Arc<Config>,
    ) -> Result<String> {
        if !is_image(&field) {
            return Err(Error::BadRequest(String::from("Not an image file")));
        }

        // get file extension
        let (_, ext) = extract_filename(&field)?;

        // create new avatar name and url
        let str7 = random_string(7);
        let now = DateTime::now().timestamp;

        let url = if ext.is_empty() {
            format!("{}/img{}-{}-{}", &config.avatar_dir, user_id, str7, now)
        } else {
            format!(
                "{}/img{}-{}-{}.{}",
                &config.avatar_dir, user_id, str7, now, ext
            )
        };

        // save stream to file
        let avatar_path = get_fullpath(&config, &url);
        stream_to_file(avatar_path, field).await?;

        Ok(url)
    }

    /// Remove all unused avatar
    ///
    pub async fn clean_avatars(user_id: i64, avatar: &str, config: Arc<Config>) -> Result<()> {
        // this pattern is associated with the avatar url
        let pat = format!("img{}-", user_id);
        let avatar_path = get_fullpath(&config, &config.avatar_dir);

        let mut entries = tokio::fs::read_dir(avatar_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with(&pat) && !avatar.contains(name) {
                    tokio::fs::remove_file(entry.path()).await?;
                }
            }
        }
        Ok(())
    }

    /// Save the uploaded file in share directory
    ///
    pub async fn save_shared_file(field: Field<'static>, config: Arc<Config>) -> Result<FileMeta> {
        // get file name from field
        let (name, ext) = extract_filename(&field)?;

        // check if the file is an image
        let img = is_image(&field);

        // generate new filename and url
        let str9 = random_string(9);
        let now = DateTime::now().timestamp;

        let url = if ext.is_empty() {
            format!("{}/s{}-{}", &config.share_dir, str9, now)
        } else {
            format!("{}/s{}-{}.{}", &config.share_dir, str9, now, ext)
        };

        // save stream to file
        let file_path = get_fullpath(&config, &url);
        stream_to_file(file_path, field).await?;

        let file_meta = FileMeta { name, url, img };
        Ok(file_meta)
    }

    /// Get the size of share directory
    ///
    pub async fn get_shared_size(config: Arc<Config>) -> Result<String> {
        let mut sz = 0_u64;

        let shared_path = get_fullpath(&config, &config.share_dir);
        let mut entries = tokio::fs::read_dir(shared_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if let Ok(metadata) = entry.metadata().await {
                sz += metadata.len();
            }
        }
        Ok(stringify_size(sz as f64))
    }

    /// Remove outdated shared files
    ///
    pub async fn clean_outdated_files(config: Arc<Config>) -> Result<String> {
        let mut sz = 0_u64;
        let now = SystemTime::now();

        let shared_path = get_fullpath(&config, &config.share_dir);
        let mut entries = tokio::fs::read_dir(shared_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if let Ok(metadata) = entry.metadata().await {
                let Ok(create_at) = metadata.created() else {
                    continue;
                };
                let Ok(dur) = now.duration_since(create_at) else {
                    continue;
                };
                if dur > config.expire_duration {
                    tokio::fs::remove_file(entry.path()).await?;
                    sz += metadata.len();
                }
            }
        }
        Ok(stringify_size(sz as f64))
    }
}

// ==================== // FileLink // ==================== //

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct FileLink {
    pub id: i64,
    pub name: String,
    pub link: String,
    pub qrlink: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileLinks {
    pub total: i32,
    pub pages: i32,
    pub filelinks: Vec<FileLink>,
}

#[cfg(feature = "ssr")]
impl FileLink {
    /// Save a file with link and qr code link
    ///
    pub async fn save(
        field: Field<'static>,
        host: String,
        store: &Store,
        config: Arc<Config>,
    ) -> Result<()> {
        // get file name from field
        let (name, ext) = extract_filename(&field)?;

        // create new file name and link
        let str7 = random_string(7);
        let key = Uuid::new_v4();
        let link = if ext.is_empty() {
            format!("{}/{}{}", &config.archive_dir, str7, key)
        } else {
            format!("{}/{}{}.{}", &config.archive_dir, str7, key, ext)
        };

        // save stream to file
        let file_path = get_fullpath(&config, &link);
        stream_to_file(file_path, field).await?;

        // convert file full link to qr code
        let full_link = format!("{}{}", host, link);
        let qrcode = QrCode::new(&full_link).map_err(|_| Error::InternalServer)?;

        // create qr image file link
        let now = DateTime::now().timestamp;
        let qrlink = format!("{}/qr{}-{}.png", &config.archive_dir, str7, now);

        // save qr image
        let image_file = qrcode.render::<Luma<u8>>().build();
        let image_path = get_fullpath(&config, &qrlink);
        image_file
            .save(image_path)
            .map_err(|_| Error::InternalServer)?;

        // save file information in database
        sqlx::query("INSERT INTO filelinks (name, link, qrlink) VALUES ($1, $2, $3)")
            .bind(&name)
            .bind(&link)
            .bind(&qrlink)
            .execute(&store.pool)
            .await?;
        Ok(())
    }

    /// Delete the file and links in database
    ///
    #[cfg(feature = "ssr")]
    pub async fn delete(link_id: i64, store: &Store, config: Arc<Config>) -> Result<()> {
        let fl: FileLink =
            sqlx::query_as("SELECT id, name, link, qrlink FROM filelinks WHERE id = $1")
                .bind(&link_id)
                .fetch_one(&store.pool)
                .await?;

        // delete file and qr image
        let file_path = get_fullpath(&config, &fl.link);
        if tokio::fs::try_exists(&file_path).await? {
            tokio::fs::remove_file(file_path).await?;
        }

        let qr_image = get_fullpath(&config, &fl.qrlink);
        if tokio::fs::try_exists(&qr_image).await? {
            tokio::fs::remove_file(qr_image).await?;
        }

        sqlx::query("DELETE FROM filelinks WHERE id = $1")
            .bind(&link_id)
            .execute(&store.pool)
            .await?;
        Ok(())
    }

    pub async fn list(page_id: i32, store: &Store) -> Result<FileLinks> {
        let offset = (page_id - 1) * 5;

        let result: Vec<ListLinksRow> = sqlx::query_as(
            "
            SELECT id, name, link, qrlink, count(*) OVER() AS total
            FROM filelinks LIMIT 5 OFFSET $1",
        )
        .bind(offset)
        .fetch_all(&store.pool)
        .await?;

        let total = match result.get(0) {
            Some(row) => row.total.unwrap_or(0),
            None => 0,
        };
        let pages = (total + 4) / 5;

        let filelinks: Vec<FileLink> = result.into_iter().map(|row| row.into()).collect();

        Ok(FileLinks {
            total,
            pages,
            filelinks,
        })
    }
}

#[cfg(feature = "ssr")]
#[derive(sqlx::FromRow)]
struct ListLinksRow {
    id: i64,
    name: String,
    link: String,
    qrlink: String,
    total: Option<i32>,
}

#[cfg(feature = "ssr")]
impl From<ListLinksRow> for FileLink {
    fn from(v: ListLinksRow) -> Self {
        Self {
            id: v.id,
            name: v.name,
            link: v.link,
            qrlink: v.qrlink,
        }
    }
}

// ==================== // FileInfo // ==================== //

#[derive(Clone, Default)]
pub struct FileInfo {
    pub name: String,
    pub size: String,
}

impl FileInfo {
    pub fn new(name: String, size: f64) -> Self {
        Self {
            name,
            size: stringify_size(size),
        }
    }
}

// ==================== // UTILS // ==================== //

#[cfg(feature = "ssr")]
async fn stream_to_file<S, E>(path: PathBuf, stream: S) -> Result<()>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    async {
        // convert the stream into an `AsyncRead`.
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures_util::pin_mut!(body_reader);

        // copy the body into the file.
        let mut file = BufWriter::new(File::create(path).await?);
        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, io::Error>(())
    }
    .await?;

    Ok(())
}

/// Get the full path of the file or directory
///
#[cfg(feature = "ssr")]
fn get_fullpath(config: &Arc<Config>, url: &str) -> PathBuf {
    if url.starts_with("/") {
        Path::new(&config.site_root).join(&url[1..])
    } else {
        Path::new(&config.site_root).join(url)
    }
}

#[cfg(feature = "ssr")]
fn random_string(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

#[cfg(feature = "ssr")]
fn is_image(field: &Field<'static>) -> bool {
    if let Some(content_type) = field.content_type() {
        if content_type.to_string().starts_with("image") {
            true
        } else {
            false
        }
    } else {
        false
    }
}

#[cfg(feature = "ssr")]
fn extract_filename(field: &Field<'static>) -> Result<(String, String)> {
    if let Some(file_name) = field.file_name() {
        let fname = Path::new(file_name.trim());

        // parse name and extension
        let name = fname
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .to_owned();
        let ext = fname
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .to_owned();

        Ok((name, ext))
    } else {
        Err(Error::BadRequest(String::from(
            "File name not found in Field",
        )))
    }
}

fn stringify_size(size: f64) -> String {
    if size < 1000.0 {
        format!("{:.1} B", size)
    } else if size < 1e6 {
        format!("{:.1} KB", size / 1000.0)
    } else if size < 1e9 {
        format!("{:.1} MB", size / 1e6)
    } else {
        format!("{:.1} GB", size / 1e9)
    }
}
