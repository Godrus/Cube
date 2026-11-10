"""Tiny PNG reader/writer (standard library only): 8-bit RGB or RGBA, non-interlaced.

Enough for Godot screenshots and for reproducible defect fixtures; not a general PNG library.
"""

import struct
import zlib

SIGNATURE = b"\x89PNG\r\n\x1a\n"


def _chunks(data):
    if data[:8] != SIGNATURE:
        raise ValueError("not a PNG file")
    pos = 8
    while pos < len(data):
        length, kind = struct.unpack(">I4s", data[pos:pos + 8])
        body = data[pos + 8:pos + 8 + length]
        yield kind, body
        pos += 12 + length


def read_png(data):
    """Returns (width, height, channels, bytearray of rows*width*channels)."""
    width = height = channels = None
    idat = []
    for kind, body in _chunks(data):
        if kind == b"IHDR":
            width, height, depth, color, _comp, _filt, interlace = struct.unpack(">IIBBBBB", body)
            if depth != 8 or color not in (2, 6) or interlace != 0:
                raise ValueError("only 8-bit RGB/RGBA non-interlaced PNG is supported")
            channels = 3 if color == 2 else 4
        elif kind == b"IDAT":
            idat.append(body)
    raw = zlib.decompress(b"".join(idat))
    stride = width * channels
    out = bytearray(height * stride)
    prev = bytearray(stride)
    pos = 0
    for y in range(height):
        ftype = raw[pos]
        line = bytearray(raw[pos + 1:pos + 1 + stride])
        pos += 1 + stride
        if ftype == 1:
            for i in range(channels, stride):
                line[i] = (line[i] + line[i - channels]) & 255
        elif ftype == 2:
            for i in range(stride):
                line[i] = (line[i] + prev[i]) & 255
        elif ftype == 3:
            for i in range(stride):
                left = line[i - channels] if i >= channels else 0
                line[i] = (line[i] + ((left + prev[i]) >> 1)) & 255
        elif ftype == 4:
            for i in range(stride):
                a = line[i - channels] if i >= channels else 0
                b = prev[i]
                c = prev[i - channels] if i >= channels else 0
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                pred = a if pa <= pb and pa <= pc else (b if pb <= pc else c)
                line[i] = (line[i] + pred) & 255
        elif ftype != 0:
            raise ValueError("bad PNG filter")
        out[y * stride:(y + 1) * stride] = line
        prev = line
    return width, height, channels, out


def write_png(width, height, channels, pixels):
    """Encodes pixels (rows * width * channels bytes) as an unfiltered PNG."""
    color = 2 if channels == 3 else 6
    stride = width * channels
    raw = bytearray()
    for y in range(height):
        raw.append(0)
        raw += pixels[y * stride:(y + 1) * stride]

    def chunk(kind, body):
        crc = zlib.crc32(kind + body) & 0xFFFFFFFF
        return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", crc)

    return (
        SIGNATURE
        + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, color, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(bytes(raw), 9))
        + chunk(b"IEND", b"")
    )


def dimensions(data):
    """(width, height) of a PNG without decoding it."""
    for kind, body in _chunks(data):
        if kind == b"IHDR":
            return struct.unpack(">II", body[:8])
    raise ValueError("no IHDR")
