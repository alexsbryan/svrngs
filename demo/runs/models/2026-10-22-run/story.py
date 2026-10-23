#!/usr/bin/env python3
"""The story's stock clients (row models-run (a)): the OpenAI SDK and the
MCP SDK, nothing written for the pantry. Each subcommand prints one JSON
answer on stdout; drive.py saves it under data/responses/.

    story.py chat       --pantry URL --model ana/coder
    story.py transcribe --pantry URL --model sam/whisper-1 --audio FILE
    story.py speak      --pantry URL --model sam/kokoro --save FILE
    story.py bridge     --bridge URL --model ana/coder --shot FILE
    story.py tool       --origin http://label.localhost:PORT/
    story.py asleep     --pantry URL --model sam/whisper-1
    story.py limit      --pantry URL --model sam/kokoro
    story.py dee        --dee URL
"""
import argparse
import asyncio
import base64
import hashlib
import json
import sys

from openai import OpenAI


def client(base):
    return OpenAI(base_url=base.rstrip("/") + "/v1", api_key="the-pantry-needs-no-key")


def headers(res):
    return {k.lower(): v for k, v in dict(res.headers).items()}


def chat(args):
    c = client(args.pantry)
    parts, text = [], ""
    stream = c.chat.completions.create(
        model=args.model,
        messages=[{"role": "user", "content": args.prompt or "Reply with the single word: ok"}],
        stream=True, max_tokens=args.max_tokens,
    )
    for chunk in stream:
        parts.append(chunk.model_dump(mode="json"))
        for choice in chunk.choices:
            text += choice.delta.content or ""
    return {"status": 200, "text": text, "chunks": len(parts), "first": parts[0] if parts else None}


def once(args):
    c = client(args.pantry)
    said = c.chat.completions.create(
        model=args.model,
        messages=[{"role": "user", "content": args.prompt or "Reply with the single word: ok"}],
        max_tokens=args.max_tokens, stream=False,
    )
    return {"status": 200, "body": said.model_dump(mode="json")}


def transcribe(args):
    c = client(args.pantry)
    with open(args.audio, "rb") as f:
        said = c.audio.transcriptions.create(model=args.model, file=f)
    return {"status": 200, "body": said.model_dump(mode="json")}


def speak(args):
    c = client(args.pantry)
    res = c.audio.speech.with_raw_response.create(
        model=args.model, voice=args.voice, input=args.input, response_format="mp3")
    data = res.read()
    if args.save:
        open(args.save, "wb").write(data)
    return {"status": res.status_code, "headers": headers(res),
            "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}


def bridge(args):
    c = client(args.bridge)
    shot = base64.b64encode(open(args.shot, "rb").read()).decode()
    res = c.chat.completions.with_raw_response.create(
        model=args.model,
        messages=[{"role": "user", "content": [
            {"type": "text", "text": "This page is broken. Say what is wrong and give the fix."},
            {"type": "image_url", "image_url": {"url": "data:image/png;base64," + shot}},
        ]}],
        max_tokens=600, stream=False,
    )
    said = res.parse()
    return {"status": res.status_code, "headers": headers(res),
            "body": said.model_dump(mode="json")}


def tool(args):
    from mcp import ClientSession
    from mcp.client.streamable_http import streamablehttp_client
    o = args.origin.rstrip("/")
    scheme, rest = o.split("://", 1)
    host, port = rest.rsplit(":", 1)

    async def run():
        async with streamablehttp_client(f"http://127.0.0.1:{port}/mcp",
                                         headers={"Host": host}) as (r, w, _):
            async with ClientSession(r, w) as s:
                out = {"initialize": (await s.initialize()).model_dump(mode="json")}
                out["tools"] = (await s.list_tools()).model_dump(mode="json")
                out["whoami"] = (await s.call_tool("whoami", {})).model_dump(mode="json")
                return out
    return asyncio.run(run())


def asleep(args):
    c = client(args.pantry)
    try:
        said = c.chat.completions.create(model=args.model,
                                         messages=[{"role": "user", "content": "hi"}],
                                         max_tokens=4, stream=False)
        return {"status": 200, "substituted": True, "body": said.model_dump(mode="json")}
    except Exception as e:
        res = getattr(e, "response", None)
        return {"status": getattr(e, "status_code", None),
                "headers": headers(res) if res is not None else {},
                "body": getattr(e, "body", None), "text": str(e)}


def limit(args):
    import threading
    out = [None, None]

    def ask(i):
        c = client(args.pantry)
        try:
            res = c.audio.speech.with_raw_response.create(
                model=args.model, voice=args.voice, input=args.input, response_format="mp3")
            out[i] = {"status": res.status_code, "headers": headers(res), "bytes": len(res.read())}
        except Exception as e:
            res = getattr(e, "response", None)
            out[i] = {"status": getattr(e, "status_code", None),
                      "headers": headers(res) if res is not None else {},
                      "body": getattr(e, "body", None), "text": str(e)}
    threads = [threading.Thread(target=ask, args=(i,)) for i in range(2)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    return {"both": out}


def dee(args):
    return once(argparse.Namespace(pantry=args.dee, model="dee/teach",
                                   prompt="Say what this adapter taught you.", max_tokens=80))


def main():
    p = argparse.ArgumentParser()
    sub = p.add_subparsers(dest="do", required=True)
    a = sub.add_parser("chat"); a.add_argument("--pantry"); a.add_argument("--model", default="ana/coder"); a.add_argument("--prompt"); a.add_argument("--max-tokens", type=int, default=40); a.set_defaults(fn=chat)
    a = sub.add_parser("once"); a.add_argument("--pantry"); a.add_argument("--model", default="ana/coder"); a.add_argument("--prompt"); a.add_argument("--max-tokens", type=int, default=40); a.set_defaults(fn=once)
    a = sub.add_parser("transcribe"); a.add_argument("--pantry"); a.add_argument("--model", default="sam/whisper-1"); a.add_argument("--audio", required=True); a.set_defaults(fn=transcribe)
    a = sub.add_parser("speak"); a.add_argument("--pantry"); a.add_argument("--model", default="sam/kokoro"); a.add_argument("--voice", default="af_heart"); a.add_argument("--input", default="The pantry is open."); a.add_argument("--save"); a.set_defaults(fn=speak)
    a = sub.add_parser("bridge"); a.add_argument("--bridge"); a.add_argument("--model", default="ana/coder"); a.add_argument("--shot", required=True); a.set_defaults(fn=bridge)
    a = sub.add_parser("tool"); a.add_argument("--origin", required=True); a.set_defaults(fn=tool)
    a = sub.add_parser("asleep"); a.add_argument("--pantry"); a.add_argument("--model", default="sam/whisper-1"); a.set_defaults(fn=asleep)
    a = sub.add_parser("limit"); a.add_argument("--pantry"); a.add_argument("--model", default="sam/kokoro"); a.add_argument("--voice", default="af_heart"); a.add_argument("--input", default="The pantry is open. " * 20); a.set_defaults(fn=limit)
    a = sub.add_parser("dee"); a.add_argument("--dee", required=True); a.set_defaults(fn=dee)
    args = p.parse_args()
    json.dump(args.fn(args), sys.stdout, indent=2, default=str, ensure_ascii=False)


if __name__ == "__main__":
    main()
