//! Scratch experiment: existing transport primitives, fake live policy only.
use axum::body::{Body, Bytes, HttpBody};
use http_body_util::BodyExt;
use hyper::body::Frame;
use hyper_util::rt::TokioIo;
use n0_future::task::AbortOnDropHandle;
use std::{
    convert::Infallible,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UdpSocket},
    sync::{mpsc, oneshot, watch},
    time::{interval, sleep, timeout, Instant},
};

const SWEEP: Duration = Duration::from_secs(1);
const BOUND: Duration = Duration::from_millis(1250);

async fn revoked(policy: watch::Receiver<bool>, planted: bool) {
    if planted {
        std::future::pending::<()>().await;
    }
    let mut every = interval(SWEEP);
    while *policy.borrow() {
        every.tick().await;
    }
}

async fn guarded(work: impl Future<Output = ()>, policy: watch::Receiver<bool>, planted: bool) -> bool {
    tokio::select! {
        () = work => false,
        () = revoked(policy, planted) => true,
    }
}

struct Endless;
impl HttpBody for Endless {
    type Data = Bytes;
    type Error = Infallible;
    fn poll_frame(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Result<Frame<Bytes>, Infallible>>> {
        Poll::Ready(Some(Ok(Frame::data(Bytes::from_static(&[7; 16384])))))
    }
}

// The body owns the pump's lifetime; the pump owns the actual upstream driver.
// An independent cancellation receiver wins over any already queued frame.
struct PumpBody {
    frames: mpsc::Receiver<Result<Frame<Bytes>, hyper::Error>>,
    revoked: oneshot::Receiver<()>,
    _task: AbortOnDropHandle<()>,
}
impl HttpBody for PumpBody {
    type Data = Bytes;
    type Error = std::io::Error;
    fn poll_frame(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        if matches!(Pin::new(&mut self.revoked).poll(cx), Poll::Ready(Ok(()))) {
            self.frames.close();
            return Poll::Ready(Some(Err(std::io::Error::other("scope withdrawn"))));
        }
        self.frames
            .poll_recv(cx)
            .map(|f| f.map(|r| r.map_err(std::io::Error::other)))
    }
}

async fn http_body(planted: bool) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (closed, observed) = oneshot::channel();
    let server = AbortOnDropHandle::new(tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        let service = hyper::service::service_fn(|_: hyper::Request<hyper::body::Incoming>| async {
            Ok::<_, Infallible>(hyper::Response::new(Endless))
        });
        let _ = hyper::server::conn::http1::Builder::new()
            .serve_connection(TokioIo::new(tcp), service)
            .await;
        let _ = closed.send(());
    }));
    let tcp = TcpStream::connect(addr).await.unwrap();
    let (mut sender, driver) = hyper::client::conn::http1::handshake(TokioIo::new(tcp)).await.unwrap();
    let driver = AbortOnDropHandle::new(tokio::spawn(async move {
        let _ = driver.await;
    }));
    let request = hyper::Request::builder()
        .uri("/")
        .header("host", "localhost")
        .body(Body::empty())
        .unwrap();
    let mut upstream = sender.send_request(request).await.unwrap().into_body();
    let (policy, current) = watch::channel(true);
    let (tx, frames) = mpsc::channel(1);
    let (stopped, revoked) = oneshot::channel();
    let (blocked, ready) = oneshot::channel();
    let task = tokio::spawn(async move {
        let _driver = driver;
        let work = async move {
            let mut blocked = Some(blocked);
            while let Some(frame) = upstream.frame().await {
                if tx.capacity() == 0 {
                    if let Some(signal) = blocked.take() {
                        let _ = signal.send(());
                    }
                }
                if tx.send(frame).await.is_err() {
                    break;
                }
            }
        };
        if guarded(work, current, planted).await {
            let _ = stopped.send(());
        }
    });
    let mut body = Body::new(PumpBody {
        frames,
        revoked,
        _task: AbortOnDropHandle::new(task),
    });
    // Do not poll downstream at all: prove that the pump reaches backpressure.
    timeout(Duration::from_secs(3), ready).await.unwrap().unwrap();
    sleep(Duration::from_millis(20)).await;
    let began = Instant::now();
    policy.send(false).unwrap();
    let ended = timeout(BOUND, observed).await;
    println!(
        "body: upstream closure with unpolled/backpressured downstream: {:?}, elapsed {:?}",
        ended,
        began.elapsed()
    );
    assert!(
        matches!(ended, Ok(Ok(()))),
        "upstream must close inside 1250 ms even when body is unpolled"
    );
    assert!(body.frame().await.unwrap().is_err(), "withdrawal must beat queued data");
    drop((body, server, sender));
}

// Same copy_bidirectional future as upgraded WebSocket and raw TCP carriage.
// Byte streams are loopback TCP; this experiment does not test a WS handshake.
async fn pair() -> (TcpStream, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let client = TcpStream::connect(listener.local_addr().unwrap()).await.unwrap();
    let (server, _) = listener.accept().await.unwrap();
    (client, server)
}

async fn splice(planted: bool) {
    let (mut asker, mut a) = pair().await;
    let (mut service, mut s) = pair().await;
    let (mut unrelated, mut other) = pair().await;
    let (policy, current) = watch::channel(true);
    let task = AbortOnDropHandle::new(tokio::spawn(async move {
        guarded(
            async move {
                let _ = tokio::io::copy_bidirectional(&mut a, &mut s).await;
            },
            current,
            planted,
        )
        .await;
    }));
    asker.write_all(b"before").await.unwrap();
    let mut bytes = [0; 6];
    service.read_exact(&mut bytes).await.unwrap();
    assert_eq!(&bytes, b"before");
    sleep(Duration::from_millis(20)).await;
    let began = Instant::now();
    policy.send(false).unwrap();
    let closed = timeout(BOUND, asker.read(&mut bytes)).await;
    println!("splice: peer EOF: {:?}, elapsed {:?}", closed, began.elapsed());
    assert!(matches!(closed, Ok(Ok(0))), "upgraded copy must close inside 1250 ms");
    unrelated.write_all(b"after!").await.unwrap();
    other.read_exact(&mut bytes).await.unwrap();
    assert_eq!(&bytes, b"after!");
    println!("splice: unrelated TCP channel still carries bytes");
    drop(task);
}

async fn raw(planted: bool) {
    let (program, mut lender) = pair().await;
    let (socket, mut echo) = pair().await;
    let (policy, current) = watch::channel(true);
    let task = AbortOnDropHandle::new(tokio::spawn(async move {
        // JoinSet owns children; leaving the guarded parent drops all sockets.
        let work = async move {
            let mut children = tokio::task::JoinSet::new();
            children.spawn(async move {
                let (mut program, mut socket) = (program, socket);
                let _ = tokio::io::copy_bidirectional(&mut program, &mut socket).await;
            });
            let _ = children.join_next().await;
        };
        guarded(work, current, planted).await;
    }));
    lender.write_all(b"before").await.unwrap();
    let mut bytes = [0; 6];
    echo.read_exact(&mut bytes).await.unwrap();
    sleep(Duration::from_millis(20)).await;
    let began = Instant::now();
    policy.send(false).unwrap();
    let closed = timeout(BOUND, echo.read(&mut bytes)).await;
    println!(
        "raw: owned child socket EOF: {:?}, elapsed {:?}",
        closed,
        began.elapsed()
    );
    assert!(matches!(closed, Ok(Ok(0))), "raw children must close inside 1250 ms");
    drop(task);
}

struct Flow {
    socket: UdpSocket,
    closed: Option<oneshot::Sender<()>>,
}
impl Drop for Flow {
    fn drop(&mut self) {
        let _ = self.closed.take().unwrap().send(());
    }
}

async fn udp(planted: bool) {
    let echo = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    socket.connect(echo.local_addr().unwrap()).await.unwrap();
    let (closed, observed) = oneshot::channel();
    let flow = Flow {
        socket,
        closed: Some(closed),
    };
    let (policy, current) = watch::channel(true);
    let (packets, mut received) = mpsc::channel::<Vec<u8>>(1);
    let task = AbortOnDropHandle::new(tokio::spawn(async move {
        let work = async move {
            let mut children = tokio::task::JoinSet::new();
            children.spawn(async move {
                let flow = flow;
                while let Some(packet) = received.recv().await {
                    flow.socket.send(&packet).await.unwrap();
                }
            });
            let _ = children.join_next().await;
        };
        guarded(work, current, planted).await;
    }));
    packets.send(b"before".to_vec()).await.unwrap();
    let mut bytes = [0; 16];
    assert_eq!(echo.recv(&mut bytes).await.unwrap(), 6);
    sleep(Duration::from_millis(20)).await;
    let began = Instant::now();
    policy.send(false).unwrap();
    let ended = timeout(BOUND, observed).await;
    println!("udp: owned flow dropped: {:?}, elapsed {:?}", ended, began.elapsed());
    assert!(matches!(ended, Ok(Ok(()))), "UDP child must be dropped inside 1250 ms");
    assert!(packets.send(b"after".to_vec()).await.is_err());
    assert!(timeout(Duration::from_millis(50), echo.recv(&mut bytes)).await.is_err());
    println!("udp: closed flow receives no post-withdrawal packet");
    drop(task);
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let plant = std::env::args().nth(1).unwrap();
    println!("plant={plant}; sweep=1000 ms; experimental ceiling=1250 ms");
    http_body(plant == "body").await;
    splice(plant == "splice").await;
    raw(plant == "raw").await;
    udp(plant == "udp").await;
    println!("passed: independent body/upgrade/raw cancellation primitives");
}
