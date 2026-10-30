// usage: node - <action> ; actions: setup | login-add | login-list. Talks to 127.0.0.1:3001 from inside the container.
const { io } = require("socket.io-client");
const s = io("http://127.0.0.1:3001", { transports: ["websocket"], reconnection: false });
const U = "pass", P = process.env.KUMA_PASS; // a scratch password, given when run
const act = process.argv[2];
const done = (x) => { console.log(JSON.stringify(x)); s.close(); process.exit(0); };
s.on("connect_error", (e) => done({ connect_error: String(e) }));
s.on("connect", () => {
  if (act === "setup") return s.emit("setup", U, P, (r) => done({ setup: r }));
  s.emit("login", { username: U, password: P, token: "" }, (r) => {
    if (!r.ok) return done({ login: r });
    if (act === "login-list") return; // the server pushes monitorList after login
    s.emit("add", { type: "http", name: "written-in-container", url: "http://127.0.0.1:3001/", method: "GET",
      interval: 60, retryInterval: 60, resendInterval: 0, maxretries: 0, timeout: 48, notificationIDList: {},
      accepted_statuscodes: ["200-299"], ignoreTls: false, upsideDown: false, maxredirects: 10,
      expiryNotification: false, dns_resolve_type: "A", dns_resolve_server: "1.1.1.1", kafkaProducerBrokers: [],
      kafkaProducerSaslOptions: { mechanism: "None" } }, (a) => done({ login: r.ok, add: a }));
  });
  s.on("monitorList", (l) => { if (act === "login-list") done({ monitors: Object.values(l).map((m) => m.name) }); });
});
setTimeout(() => done({ timeout: act }), 15000);
