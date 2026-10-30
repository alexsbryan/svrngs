// usage: podman exec -i -e KUMA_PASS -w /app <container> node - < kuma_setup.js
// Adapted from ../../2026-10-30-tutorial-pass/mac/kuma/kuma.js: the first
// account, then two HTTP monitors of the pod's python http.server, every 20 s
// (Kuma's minimum). Prints ok flags only, never the password or token.
const { io } = require("socket.io-client");
const s = io("http://127.0.0.1:3001", { transports: ["websocket"], reconnection: false });
const U = "churn", P = process.env.KUMA_PASS;
const done = (x) => { console.log(JSON.stringify(x)); s.close(); process.exit(0); };
const monitor = (name, url) => ({ type: "http", name, url, method: "GET",
  interval: 20, retryInterval: 20, resendInterval: 0, maxretries: 0, timeout: 16, notificationIDList: {},
  accepted_statuscodes: ["200-299"], ignoreTls: false, upsideDown: false, maxredirects: 10,
  expiryNotification: false, dns_resolve_type: "A", dns_resolve_server: "1.1.1.1", kafkaProducerBrokers: [],
  kafkaProducerSaslOptions: { mechanism: "None" } });
s.on("connect_error", (e) => done({ connect_error: String(e) }));
s.on("connect", () => {
  s.emit("setup", U, P, (r) => {
    s.emit("login", { username: U, password: P, token: "" }, (l) => {
      if (!l.ok) return done({ setup: r.ok, login: l.ok, msg: l.msg });
      s.emit("add", monitor("target-root", "http://127.0.0.1:8000/"), (a) => {
        s.emit("add", monitor("target-listing", "http://127.0.0.1:8000/?page=2"), (b) =>
          done({ setup: r.ok, login: l.ok, add1: a.ok, add2: b.ok, ids: [a.monitorID, b.monitorID] }));
      });
    });
  });
});
setTimeout(() => done({ timeout: true }), 20000);
