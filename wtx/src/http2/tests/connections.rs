use crate::{
  http::{Headers, Method, ReqResBuffer, ReqResData, ReqUri, Request, StatusCode},
  http2::{Http2Buffer, Http2ErrorCode, Http2Params, Http2Tokio},
  misc::{UriRef, UriString, _uri},
  rng::StaticRng,
};
use core::time::Duration;
use tokio::net::{TcpListener, TcpStream};

#[tokio::test]
async fn connections() {
  #[cfg(feature = "_tracing-subscriber")]
  let _rslt = crate::misc::tracing_subscriber_init();
  let uri = _uri();
  server(&uri).await;
  client(uri).await;
}

async fn client(uri: UriString) {
  let mut rrb = ReqResBuffer::default();
  rrb.headers_mut().set_max_bytes(6);
  rrb.headers_mut().reserve(6, 1).unwrap();
  let mut client = Http2Tokio::connect(
    Http2Buffer::new(StaticRng::default()),
    Http2Params::default(),
    TcpStream::connect(uri.host()).await.unwrap(),
  )
  .await
  .unwrap();

  let uri_ref = uri.to_ref();

  rrb = stream_client(&mut client, rrb, &uri_ref).await;
  _0(rrb.body(), rrb.headers());

  rrb.clear();
  rrb.headers_mut().push_front((b"123", b"456").into(), &[]).unwrap();
  rrb = stream_client(&mut client, rrb, &uri_ref).await;
  _1(rrb.body(), rrb.headers());

  rrb.clear();
  rrb.extend_body(b"123").unwrap();
  rrb = stream_client(&mut client, rrb, &uri_ref).await;
  _2(rrb.body(), rrb.headers());

  rrb.clear();
  rrb.extend_body(b"123").unwrap();
  rrb.headers_mut().push_front((b"123", b"456").into(), &[]).unwrap();
  rrb = stream_client(&mut client, rrb, &uri_ref).await;
  _3(rrb.body(), rrb.headers());

  client.send_go_away(Http2ErrorCode::NoError).await;

  tokio::time::sleep(Duration::from_millis(100)).await;
}

async fn server(uri: &UriString) {
  let listener = TcpListener::bind(uri.host()).await.unwrap();
  let _server_jh = tokio::spawn(async move {
    let (stream, _) = listener.accept().await.unwrap();
    let mut rrb = ReqResBuffer::default();
    let mut server =
      Http2Tokio::accept(Http2Buffer::new(StaticRng::default()), Http2Params::default(), stream)
        .await
        .unwrap();

    rrb = stream_server(&mut server, rrb, |req| {
      _0(req.rrd.body(), req.rrd.headers());
    })
    .await;
    rrb = stream_server(&mut server, rrb, |req| {
      _1(req.rrd.body(), req.rrd.headers());
    })
    .await;
    rrb = stream_server(&mut server, rrb, |req| {
      _2(req.rrd.body(), req.rrd.headers());
    })
    .await;
    let _rrb = stream_server(&mut server, rrb, |req| {
      _3(req.rrd.body(), req.rrd.headers());
    })
    .await;
  });
}

async fn stream_server(
  server: &mut Http2Tokio<Http2Buffer<ReqResBuffer>, ReqResBuffer, TcpStream, false>,
  rrb: ReqResBuffer,
  mut cb: impl FnMut(Request<&mut ReqResBuffer>),
) -> ReqResBuffer {
  loop {
    let mut stream = server.stream(rrb).await.unwrap();
    let (mut req_rrb, method) = stream.recv_req().await.unwrap();
    cb(req_rrb.as_http2_request_mut(method));
    stream.send_res(req_rrb.as_http2_response(StatusCode::Ok)).await.unwrap();
    break req_rrb;
  }
}

async fn stream_client(
  client: &mut Http2Tokio<Http2Buffer<ReqResBuffer>, ReqResBuffer, TcpStream, true>,
  rrb: ReqResBuffer,
  uri: &UriRef<'_>,
) -> ReqResBuffer {
  let mut stream = client.stream().await.unwrap();
  stream.send_req(rrb.as_http2_request(Method::Get), ReqUri::Param(uri)).await.unwrap();
  stream.recv_res(rrb).await.unwrap().0
}

#[track_caller]
fn _0(body: &[u8], headers: &Headers) {
  assert_eq!((body.len(), headers.bytes_len(), headers.elements_len()), (0, 0, 0));
}
#[track_caller]
fn _1(body: &[u8], headers: &Headers) {
  assert_eq!((body.len(), headers.bytes_len(), headers.elements_len()), (0, 6, 1));
}
#[track_caller]
fn _2(body: &[u8], headers: &Headers) {
  assert_eq!((body.len(), headers.bytes_len(), headers.elements_len()), (3, 0, 0));
}
#[track_caller]
fn _3(body: &[u8], headers: &Headers) {
  assert_eq!((body.len(), headers.bytes_len(), headers.elements_len()), (3, 6, 1));
}
