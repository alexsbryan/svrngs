#!/bin/bash
# q3-write.sh <hostport> <container-to-restart> <state-volume> <email>
# One write in the container (sign up, the first account; then a chat), proof it
# is in the state volume, and that it survives `podman restart`.
P=$1; C=$2; V=$3; EM=$4; PW=svrngs-pass-only
api() { curl -s -m 20 -H 'Content-Type: application/json' ${TOK:+-H "Authorization: Bearer $TOK"} "$@"; }
echo "## signup at :$P"
TOK=$(api -X POST http://127.0.0.1:$P/api/v1/auths/signup -d "{\"name\":\"Pass\",\"email\":\"$EM\",\"password\":\"$PW\"}" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("token") or ""); print("role:", d.get("role"), file=sys.stderr)')
[ -n "$TOK" ] && echo "token: received (not printed)" || echo "token: NONE"
echo "## a chat"
api -X POST http://127.0.0.1:$P/api/v1/chats/new -d '{"chat":{"title":"svrngs-pass chat","messages":[]}}' | python3 -c 'import json,sys; d=json.load(sys.stdin); print("chat id:", d.get("id"), "title:", d.get("title"))'
echo "## in the volume $V (read from a copy of its files, read-only)"
M=$(podman volume inspect --format '{{.Mountpoint}}' $V)
T=$(mktemp -d); cp $M/webui.db* $T/ 2>/dev/null
python3 - "$T/webui.db" "$EM" <<'PY'
import sqlite3,sys
c=sqlite3.connect(sys.argv[1])
print("users with that email:", c.execute("select count(*) from user where email=?",(sys.argv[2],)).fetchone()[0])
print("chats titled svrngs-pass chat:", c.execute("select count(*) from chat where title='svrngs-pass chat'").fetchone()[0])
PY
rm -rf $T
echo "## podman restart $C"; podman restart $C >/dev/null
for i in $(seq 90); do curl -s -o /dev/null -m 3 http://127.0.0.1:$P/health && break; sleep 2; done
echo "## after restart: sign in with the same account, list chats"
TOK=$(api -X POST http://127.0.0.1:$P/api/v1/auths/signin -d "{\"email\":\"$EM\",\"password\":\"$PW\"}" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("token") or "")')
[ -n "$TOK" ] && echo "signin: ok" || echo "signin: FAILED"
api http://127.0.0.1:$P/api/v1/chats/ | python3 -c 'import json,sys; d=json.load(sys.stdin); print("chats:", [x.get("title") for x in d])'
