use crate::repo::{EntryKind, EntryPath, Result, Snapshot};
use askama_axum::{IntoResponse, Response};
use axum::{body::Body, http::header, routing::get, Router};
use futures_core::Stream;
use tempfile::{tempdir_in, TempDir};
use tokio::{fs::File, io::BufReader};
use tokio_util::io::ReaderStream;

pub fn routes() -> Router<()> {
    Router::new().route("/:repo/:snapshot/*path", get(download))
}

// TODO Execute battle plan:
//
// - The code below is mostly sync and stores a bunch of temporary files anyway
// - Make downloads a glorified share-link by using the same logic under-the-hood
//   Instead of immediately showing a progress bar followed by a link and QR code or whatever:
//      1. Dispatch the restore
//      2. Wait a couple seconds to see whether it becomes ready
//      3. Redirect to the progress page if it does not become ready or start download stream if it does
// - Advantage of efficiently resumable downloads for large files
//
// - Central restore manager struct
// - Jobs can be submitted (Snapshot+Path+Expiry)
// - Restore Job ID is returned
// - Progress updates can be fetched or subscribed to
// - Files will be restored as-is
// - Directories will be restored and then put into a single-file archive
// - Should probably be mostly sync and have its own thread(s) so we don't annoy tokio
//     - Rustic is not async so dealing with it would be utter pain
pub async fn download(snapshot: Snapshot, EntryPath(path): EntryPath) -> Result<Response> {
    let entry = snapshot.entry(&path)?;
    let name = entry
        .path
        .file_name()
        .expect("attempted to restore entry without a name");

    let destination = tempdir_in("/tmp/restic")?;
    let details = snapshot.restore(&path, &destination);
    println!("Restore complete: {details:?}");

    let entries = std::fs::read_dir(&destination)?
        .map(|res| res.map(|e| e.path()))
        .collect::<std::result::Result<Vec<_>, std::io::Error>>()?;

    dbg!(entries);

    let response = match entry.kind {
        EntryKind::File => {
            let body = Body::from_stream(TemporaryDownloadStream {
                stream: ReaderStream::new(BufReader::new(
                    File::open(destination.path().join(name)).await?,
                )),
                _source: destination,
            });

            (
                [
                    (header::CONTENT_TYPE, "application/octet-stream"),
                    (
                        header::CONTENT_DISPOSITION,
                        &format!(r#"attachment; filename="{}""#, name.to_string_lossy()),
                    ),
                ],
                body,
            )
                .into_response()
        }
        EntryKind::Directory => {
            let tempdir = tempdir_in("/tmp/restic")?;
            let archive_path = tempdir.path().join("archive.tar");

            println!("Building archive");
            let mut builder = tar::Builder::new(std::fs::File::create_new(&archive_path)?);

            println!("Appending files at {name:?} from {destination:?}");
            builder.append_dir_all(name, destination)?;

            println!("Finishing archive");
            builder.finish()?;

            println!("Streaming archive");
            let body = Body::from_stream(TemporaryDownloadStream {
                stream: ReaderStream::new(BufReader::new(File::open(archive_path).await?)),
                _source: tempdir,
            });

            (
                [
                    (header::CONTENT_TYPE, "application/x-tar"),
                    (
                        header::CONTENT_DISPOSITION,
                        &format!(r#"attachment; filename="{}.tar""#, name.to_string_lossy()),
                    ),
                ],
                body,
            )
                .into_response()
        }
    };

    Ok(response)
}

struct TemporaryDownloadStream<S: Stream + Unpin> {
    _source: TempDir,
    stream: S,
}

impl<S: Stream + Unpin> Stream for TemporaryDownloadStream<S> {
    type Item = S::Item;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut pinned = std::pin::pin!(&mut self.stream);
        pinned.as_mut().poll_next(cx)
    }
}
