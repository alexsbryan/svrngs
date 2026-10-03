// The host's loopback append (crates/svrngs/src/bridge.rs), as the bridge
// calls it: every request carries the capability token. A speaker the guest
// book removed comes back as `{removed: true}`; every other refusal throws.

export function hostClient({ url, token }) {
  async function call(method, path, body) {
    const res = await fetch(new URL(path, url), {
      method,
      headers: { authorization: `Bearer ${token}`, 'content-type': 'application/json' },
      body: body === undefined ? undefined : JSON.stringify(body),
    });
    const json = await res.json().catch(() => ({ error: `a ${res.status} with no JSON body` }));
    if (res.status === 403 && json.removed) return { removed: true, error: json.error };
    if (!res.ok) throw new Error(`${method} ${path}: ${res.status} ${json.error}`);
    return json;
  }
  return {
    group: () => call('GET', '/group'),
    guest: (g) => call('POST', '/guest', g),
    append: (a) => call('POST', '/append', a),
  };
}
