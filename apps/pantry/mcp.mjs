// MCP over Streamable HTTP, as much as discovery needs: initialize, then
// tools/list. No session is kept beyond one ask, and nothing is interpreted
// but the protocol's own answer.

const PROTOCOL = '2025-06-18';

async function post(url, message, { session, version, timeout, token }) {
  const headers = {
    accept: 'application/json, text/event-stream',
    'content-type': 'application/json',
  };
  if (token) headers.authorization = `Bearer ${token}`;
  if (session) headers['mcp-session-id'] = session;
  if (version) headers['mcp-protocol-version'] = version;
  const res = await fetch(url, {
    method: 'POST',
    headers,
    body: JSON.stringify(message),
    signal: AbortSignal.timeout(timeout),
  });
  const text = await res.text();
  const sessionId = res.headers.get('mcp-session-id') || session;
  if (!res.ok) throw new Error(`MCP answered ${res.status}${text ? `: ${text.slice(0, 200)}` : ''}`);
  if (!text.trim()) return { session: sessionId };
  return { session: sessionId, message: parse(text, res.headers.get('content-type') || '') };
}

// A Streamable HTTP answer is JSON, or one or more SSE events carrying it.
function parse(text, contentType) {
  if (contentType.includes('text/event-stream')) {
    for (const block of text.split(/\r?\n\r?\n/)) {
      const data = block
        .split(/\r?\n/)
        .filter((line) => line.startsWith('data:'))
        .map((line) => line.slice(5).trim())
        .join('\n');
      if (!data) continue;
      try {
        const value = JSON.parse(data);
        if (value && value.id !== undefined) return value;
      } catch {
        // another event's data, or the protocol's keep-alive; keep reading
      }
    }
    throw new Error('MCP answered an event stream with no reply');
  }
  return JSON.parse(text);
}

/// The tools a Streamable HTTP MCP server at `origin` answers `tools/list`
/// with, annotations included as the server sent them. Each ask waits up to
/// `timeout` ms, the caller's one probe time. A lend's origin on this
/// computer answers a program only with this computer's `token`, which
/// svrngs takes off before the lend's service is asked (give-session).
export async function toolsAt(origin, timeout, token) {
  const url = `${origin.replace(/\/+$/, '')}/mcp`;
  const init = await post(url, {
    jsonrpc: '2.0',
    id: 1,
    method: 'initialize',
    params: {
      protocolVersion: PROTOCOL,
      capabilities: {},
      clientInfo: { name: 'pantry', version: '0' },
    },
  }, { timeout, token });
  const version = init.message?.result?.protocolVersion || PROTOCOL;
  await post(url, { jsonrpc: '2.0', method: 'notifications/initialized' }, {
    session: init.session,
    version,
    timeout,
    token,
  });
  const listed = await post(url, { jsonrpc: '2.0', id: 2, method: 'tools/list', params: {} }, {
    session: init.session,
    version,
    timeout,
    token,
  });
  if (listed.message?.error) {
    throw new Error(`MCP tools/list: ${listed.message.error.message || 'refused'}`);
  }
  const tools = listed.message?.result?.tools;
  if (!Array.isArray(tools)) throw new Error('MCP tools/list answered no tools');
  return tools.map((tool) => ({
    name: String(tool.name),
    effect: effectOf(tool.annotations),
  }));
}

/// What a tool does, from the server's own annotations. No annotation says
/// nothing: it is never read as read-only.
function effectOf(annotations) {
  if (!annotations || typeof annotations !== 'object') return 'not declared';
  if (annotations.readOnlyHint === true) return 'read-only';
  if (annotations.destructiveHint === true) return 'destructive';
  return 'not declared';
}
