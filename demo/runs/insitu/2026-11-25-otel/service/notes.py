"""Session note -> claim fields. A stand-in for the vendor's parser."""
from datetime import datetime

FIELDS = {"Date of service": "date", "Client ID": "client_id", "CPT": "cpt", "Diagnosis": "icd10", "Minutes": "minutes"}


def read(raw: bytes) -> str:
    try:
        return raw.decode("utf-8")
    except UnicodeDecodeError:
        return raw.decode("cp1252")


def parse(raw: bytes) -> dict:
    form = {}
    for line in read(raw).splitlines():
        label, sep, value = line.partition(":")
        if sep and label.strip() in FIELDS:
            form[FIELDS[label.strip()]] = value.strip()
    missing = [k for k in FIELDS.values() if k not in form]
    if missing:
        raise KeyError(f"missing {missing}")
    form["date"] = datetime.strptime(form["date"], "%Y-%m-%d").date().isoformat()
    return form
