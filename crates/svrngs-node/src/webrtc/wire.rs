// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from svrn-docs rings/current/webrtc-transport-spike/carrier/src/lib.rs@0753e2d
// (Apache-2.0/MIT: the WebRTC spikes are ours, CHARTER "Reuse")
//! iroh's custom transport over any datagram wire, keyed by the peer's
//! iroh key (the transport spike's README, "Borrowed, not depended on").
//! The ideas are mbe24/iroh-webrtc-transport@229c5fd15aec's `adapter.rs`;
//! the code is ours. A wire is two things: a queue of datagrams that
//! arrive, each with the key it came from, and a function that sends one to
//! a key. Both halves know nothing of iroh, and this file knows nothing of
//! WebRTC.

use std::io;
use std::sync::Arc;
use std::task::{Context, Poll};

use iroh::endpoint::transports::{
    CustomEndpoint, CustomSender, CustomTransport, RecvInfo, Transmit,
};
use iroh_base::CustomAddr;
use tokio::sync::mpsc::UnboundedReceiver;

/// A peer, by its iroh key.
pub type PeerKey = [u8; 32];

/// The custom transport's id: a `CustomAddr` with it carries a [`PeerKey`].
pub const TRANSPORT_ID: u64 = 0x7376_7277; // "svrw"

/// Sends one datagram to a peer, best effort: a wire with no open
/// channel to it, or a full one, drops it, and QUIC above retransmits.
pub type Send = Arc<dyn Fn(&PeerKey, &[u8]) + std::marker::Send + Sync>;

/// The custom address of `key` on this transport.
pub fn addr_of(key: &PeerKey) -> CustomAddr {
    CustomAddr::from_parts(TRANSPORT_ID, key)
}

/// The wire's two halves: the queue of datagrams that arrived, and the
/// function that sends one.
type Halves = (UnboundedReceiver<(PeerKey, Vec<u8>)>, Send);

/// The transport iroh is handed (`Endpoint::builder(..).add_custom_transport`).
/// Its halves wait here until iroh binds it, once.
pub struct Wire {
    local: CustomAddr,
    #[expect(
        clippy::disallowed_types,
        reason = "the halves iroh binds once, taken at bind; the spike's own"
    )]
    parts: std::sync::Mutex<Option<Halves>>,
}

impl Wire {
    #[expect(
        clippy::disallowed_types,
        reason = "the halves iroh binds once, taken at bind; the spike's own"
    )]
    pub fn new(
        local: &PeerKey,
        arrived: UnboundedReceiver<(PeerKey, Vec<u8>)>,
        send: Send,
    ) -> Arc<Self> {
        Arc::new(Self {
            local: addr_of(local),
            parts: std::sync::Mutex::new(Some((arrived, send))),
        })
    }
}

impl std::fmt::Debug for Wire {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Wire")
            .field("local", &self.local)
            .finish_non_exhaustive()
    }
}

impl CustomTransport for Wire {
    fn bind(&self) -> io::Result<Box<dyn CustomEndpoint>> {
        let (arrived, send) = self
            .parts
            .lock()
            .unwrap()
            .take()
            .ok_or_else(|| io::Error::other("bound already"))?;
        Ok(Box::new(Bound {
            local: self.local.clone(),
            arrived,
            send,
            watch: n0_watcher::Watchable::new(vec![self.local.clone()]),
        }))
    }
}

struct Bound {
    local: CustomAddr,
    arrived: UnboundedReceiver<(PeerKey, Vec<u8>)>,
    send: Send,
    watch: n0_watcher::Watchable<Vec<CustomAddr>>,
}

impl std::fmt::Debug for Bound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bound")
            .field("local", &self.local)
            .finish_non_exhaustive()
    }
}

impl CustomEndpoint for Bound {
    fn watch_local_addrs(&self) -> n0_watcher::Direct<Vec<CustomAddr>> {
        self.watch.watch()
    }

    fn create_sender(&self) -> Arc<dyn CustomSender> {
        Arc::new(Sender(self.send.clone()))
    }

    fn poll_recv(
        &mut self,
        cx: &mut Context,
        bufs: &mut [io::IoSliceMut<'_>],
        metas: &mut [noq_udp::RecvMeta],
        recv_infos: &mut [RecvInfo],
    ) -> Poll<io::Result<usize>> {
        let mut n = 0;
        while n < bufs.len() {
            match self.arrived.poll_recv(cx) {
                Poll::Ready(Some((from, data))) => {
                    if bufs[n].len() < data.len() {
                        continue; // larger than a slot: dropped, as a lossy path drops
                    }
                    bufs[n][..data.len()].copy_from_slice(&data);
                    metas[n].len = data.len();
                    metas[n].stride = data.len();
                    recv_infos[n] = RecvInfo::new(addr_of(&from), Some(self.local.clone()));
                    n += 1;
                }
                // Idle, or every sender gone: nothing now. Never Ok(0), which
                // iroh reads as this endpoint closing.
                Poll::Ready(None) | Poll::Pending => break,
            }
        }
        match n {
            0 => Poll::Pending,
            n => Poll::Ready(Ok(n)),
        }
    }
}

struct Sender(Send);

impl std::fmt::Debug for Sender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Sender")
    }
}

impl CustomSender for Sender {
    fn is_valid_send_addr(&self, addr: &CustomAddr) -> bool {
        addr.id() == TRANSPORT_ID
    }

    fn poll_send(
        &self,
        _: &mut Context,
        dst: &CustomAddr,
        _: Option<&CustomAddr>,
        transmit: &Transmit<'_>,
    ) -> Poll<io::Result<()>> {
        if let Ok(key) = <PeerKey>::try_from(dst.data()) {
            // A batch is `contents` cut into `segment_size` datagrams.
            let segment = transmit
                .segment_size
                .unwrap_or(transmit.contents.len())
                .max(1);
            for datagram in transmit.contents.chunks(segment) {
                (self.0)(&key, datagram);
            }
        }
        Poll::Ready(Ok(()))
    }
}
