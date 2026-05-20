use std::time::Duration;

use anyhow::Context;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::timeout;

use podctl::{Request, Response, socket_path};

const READ_TIMEOUT: Duration = Duration::from_secs(10);
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn send(req: &Request) -> anyhow::Result<Response> {
    let path = socket_path();
    let stream = UnixStream::connect(&path)
        .await
        .with_context(|| format!("connect daemon at {}", path.display()))?;
    let (rx, mut tx) = stream.into_split();

    let mut line = serde_json::to_vec(req).context("serialise request")?;
    line.push(b'\n');
    timeout(WRITE_TIMEOUT, tx.write_all(&line))
        .await
        .context("write timeout")?
        .context("write")?;
    timeout(WRITE_TIMEOUT, tx.flush())
        .await
        .context("flush timeout")?
        .context("flush")?;

    let mut reader = BufReader::new(rx);
    let mut buf = String::new();
    timeout(READ_TIMEOUT, reader.read_line(&mut buf))
        .await
        .context("read timeout")?
        .context("read")?;
    let resp: Response = serde_json::from_str(buf.trim()).context("parse response")?;
    Ok(resp)
}
