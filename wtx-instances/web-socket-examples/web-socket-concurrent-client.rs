//! Encrypted WebSocket client that reads and writes frames in different tasks.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use tokio::{net::TcpStream, sync::Mutex};
use wtx::{
  misc::{Arc, TokioRustlsConnector, Uri},
  web_socket::{Frame, OpCode, WebSocketConnector},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("ws://www.example.com");
  let connector = TokioRustlsConnector::from_auto()?.push_certs(wtx_instances::ROOT_CA)?;
  let stream = TcpStream::connect(uri.hostname_with_implied_port()).await?;
  let ws = WebSocketConnector::default()
    .connect(connector.connect_without_client_auth(uri.hostname(), stream).await?, &uri.to_ref())
    .await?;
  let mut parts = ws.into_parts::<Arc<Mutex<_>>, _, _>(|el| tokio::io::split(el));
  let reader_jh = tokio::spawn(async move {
    loop {
      let frame = parts.reader.read_frame().await?;
      match (frame.op_code(), frame.text_payload()) {
        (_, Some(elem)) => println!("{elem}"),
        (OpCode::Close, _) => break,
        _ => {}
      }
    }
    wtx::Result::Ok(())
  });
  let writer_jh = tokio::spawn(async move {
    parts.writer.write_frame(&mut Frame::new_fin(OpCode::Text, *b"Hi and Bye")).await?;
    parts.writer.write_frame(&mut Frame::new_fin(OpCode::Close, [])).await?;
    wtx::Result::Ok(())
  });
  let (reader_rslt, writer_rslt) = tokio::join!(reader_jh, writer_jh);
  reader_rslt??;
  writer_rslt??;
  Ok(())
}
