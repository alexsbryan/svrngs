"""Sam's tool: an MCP Streamable HTTP server with one read-only tool and one
destructive one, so the pantry's list has both annotations to show. It is
the run's own stand-in for a tool Sam lends; nothing here is in the pantry.
"""
import json
import os

from mcp.server.fastmcp import Context, FastMCP
from mcp.types import ToolAnnotations

mcp = FastMCP("sam-tools")


@mcp.tool(annotations=ToolAnnotations(readOnlyHint=True))
def whoami(ctx: Context) -> str:
    """Who is calling, as the door named them."""
    req = getattr(ctx.request_context, "request", None)
    headers = dict(req.headers) if req is not None else {}
    return json.dumps({k: v for k, v in headers.items()
                       if k.lower().startswith("x-svrngs")}, sort_keys=True)


@mcp.tool(annotations=ToolAnnotations(destructiveHint=True))
def erase(name: str) -> str:
    """A destructive stand-in: it writes nothing here."""
    return f"erased {name}"


if __name__ == "__main__":
    mcp.settings.host = "127.0.0.1"
    mcp.settings.port = int(os.environ.get("MCP_PORT", "8208"))
    mcp.run(transport="streamable-http")
