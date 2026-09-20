import os
import time
from pathlib import Path

from fastapi import FastAPI, Header, HTTPException, Request, Response

SECRET = os.environ.get("SMUSIC_COVER_SECRET", "")
DATA_FILE = Path(os.environ.get("SMUSIC_COVER_FILE", "/opt/smusic-cover/cover.jpg"))
RATE_LIMIT_PER_MINUTE = 20

app = FastAPI()

current_cover_bytes = b""
put_history = {}

if DATA_FILE.exists():
    current_cover_bytes = DATA_FILE.read_bytes()


def check_rate_limit(ip):
    now = time.time()
    hits = [t for t in put_history.get(ip, []) if now - t < 60]
    hits.append(now)
    put_history[ip] = hits
    if len(hits) > RATE_LIMIT_PER_MINUTE:
        raise HTTPException(status_code=429, detail="too many uploads")


@app.put("/cover")
async def upload_cover(request: Request, x_smusic_key: str = Header(default="")):
    if not SECRET or x_smusic_key != SECRET:
        raise HTTPException(status_code=401, detail="bad key")

    client_ip = request.client.host if request.client else "unknown"
    check_rate_limit(client_ip)

    body = await request.body()
    if len(body) == 0 or len(body) > 5 * 1024 * 1024:
        raise HTTPException(status_code=400, detail="bad image size")

    global current_cover_bytes
    current_cover_bytes = body

    DATA_FILE.parent.mkdir(parents=True, exist_ok=True)
    DATA_FILE.write_bytes(body)

    return {"ok": True}


@app.get("/cover")
async def get_cover():
    if not current_cover_bytes:
        raise HTTPException(status_code=404, detail="no cover yet")

    return Response(
        content=current_cover_bytes,
        media_type="image/jpeg",
        headers={"Cache-Control": "public, max-age=86400, immutable"},
    )
