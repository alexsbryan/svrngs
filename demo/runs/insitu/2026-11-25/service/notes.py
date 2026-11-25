"""Session note -> insurance form fields. A stand-in for the vendor's parser:
it reads the labelled lines a session note carries and fills the form's
fields, each with the line it came from."""

FIELDS = {"Date of service": "date", "Client ID": "client_id", "CPT": "cpt", "Diagnosis": "icd10", "Minutes": "minutes"}


def read(raw: bytes) -> str:
    return raw.decode("utf-8")


def parse(raw: bytes) -> dict:
    text = read(raw)
    form, cited = {}, {}
    for n, line in enumerate(text.splitlines(), 1):
        label, sep, value = line.partition(":")
        if sep and label.strip() in FIELDS:
            key = FIELDS[label.strip()]
            form[key] = value.strip()
            cited[key] = n
    missing = [k for k in FIELDS.values() if k not in form]
    return {"form": form, "line_of": cited, "missing": missing}


def shape(raw: bytes) -> dict:
    """What a note looks like, never what it says: safe to show support."""
    s = {
        "bytes": len(raw),
        "bom": "utf-8" if raw.startswith(b"\xef\xbb\xbf") else "utf-16" if raw[:2] in (b"\xff\xfe", b"\xfe\xff") else None,
        "newline": "crlf" if b"\r\n" in raw else "lf" if b"\n" in raw else None,
        "high_bytes": sum(b >= 0x80 for b in raw),
    }
    try:
        raw.decode("utf-8")
    except UnicodeDecodeError as e:
        s["first_invalid_byte"] = f"0x{raw[e.start]:02x}"
        s["first_invalid_at_fraction"] = round(e.start / len(raw), 2)
    return s
