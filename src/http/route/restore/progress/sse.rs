use super::fragment::ProgressFragment;
use crate::restic::restore::progress::ProgressReceiver;
use askama_axum::IntoResponse;
use axum::response::{sse::Event, Response, Sse};
use futures::{stream, StreamExt};
use futures_time::stream::StreamExt as _;
use futures_time::time::Duration;
use std::convert::Infallible;
use tokio::sync::broadcast::error::RecvError;

impl IntoResponse for ProgressReceiver {
    fn into_response(self) -> Response {
        let update_stream = stream::unfold(self.subscribe(), move |mut progress| async move {
            loop {
                match progress.recv().await.map(ProgressFragment::from) {
                    Ok(data) => return Some((data, progress)),
                    Err(RecvError::Closed) => return None,
                    Err(RecvError::Lagged(_)) => {}
                }
            }
        });

        let mut previous = ProgressFragment::default();

        let event_stream = update_stream
            // Ensure that we don't overload the client while making sure to always emit something
            //
            // `.sample` would stop emitting once the stream is closed potentially skipping over the final value
            // Hence we use a combination of buffer and filter_map to collect all values of an interval and then take the latest.
            .buffer(Duration::from_millis(100))
            .filter_map(|buffer| async move { buffer.into_iter().last() })
            // Calculate deltas in comparison to the last sent state
            .map(move |data| {
                let delta = data.delta(&previous);
                previous = data;
                delta
            })
            // Skip over empty deltas
            .filter(|data| {
                let empty = data.is_empty();
                async move { !empty }
            })
            .map(Event::from)
            // Send a `goodbye` message that reloads the page (i.e. downloads the restore)
            .chain(stream::once(async move {
                Event::default().event("reload").data(
                    r#"<script>ca.classList.add("active"); window.location.reload();</script>"#,
                )
            }))
            .map(std::result::Result::<Event, Infallible>::Ok);

        Sse::new(event_stream).into_response()
    }
}
