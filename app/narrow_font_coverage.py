#!/usr/bin/env python3
"""Narrow the display fonts' unicode-range to the scripts they are meant to draw.

Run from app/:  python3 narrow_font_coverage.py

The three CJK display faces in --space-font ship from Google Fonts claiming far
more than the script they are there for -- every one of them also claims Latin,
most claim Cyrillic, and each claims at least one of the others' scripts. Left
alone, that makes which face draws a character depend on where it happened to
land in the fallback chain, so a Chinese reader got Hangul in Chiron and a
Korean reader got Latin in Nanum Gothic.

Only Han is genuinely ambiguous: unification gave Chinese, Japanese and Korean
the same codepoints for characters their readers draw differently, so U+76F4 has
no single correct glyph, only a correct glyph for a given reader. All three
faces keep Han and --space-font orders them by UI language. Every other script
has one correct form, so it is pinned to exactly one face here and the ordering
in global.scss stops mattering for it.

Dropping whole @font-face blocks is not enough -- several subsets mix scripts
into one face (Nanum Gothic ships U+41-47 and U+ac00 in the same one), so each
unicode-range is intersected with the allowed blocks instead.

Narrowing is the safe direction: the woff2 files still hold the glyphs we no
longer claim, and a face that does not claim a codepoint is skipped, so the
browser moves on to the next family. Never widen a range past what its file
actually contains -- that buys a download and then falls through anyway.

CJK punctuation and the fullwidth forms are in all three sets rather than none,
so they behave like Han: whichever display face the UI language put first draws
them. They have to be in the display faces at all because otherwise they drop
through to --menu-font, and a heading reading 「直線」 would set the brackets in
Noto Sans and the ideographs in the display face -- two typefaces in one word.

Note these are separate codepoints from their ASCII lookalikes: fullwidth A is
U+FF21, not U+0041, so putting the fullwidth block here does not pull ordinary
Latin back into the CJK faces. It stays with Michroma.

Google Fonts does not know about any of this. Re-downloading one of these
stylesheets undoes it -- re-run this script over the fresh file.
"""

import re
import sys
from pathlib import Path

# Hangul: Jamo, Compatibility Jamo, Jamo Extended-A/B, Syllables, halfwidth jamo.
HANGUL = [
    (0x1100, 0x11FF), (0x3130, 0x318F), (0xA960, 0xA97F),
    (0xAC00, 0xD7A3), (0xD7B0, 0xD7FF), (0xFFA0, 0xFFDC),
]
# Han: Unified, Ext A, Ext B onwards, and both compatibility-ideograph blocks.
HAN = [
    (0x3400, 0x4DBF), (0x4E00, 0x9FFF), (0xF900, 0xFAFF),
    (0x20000, 0x2A6DF), (0x2A700, 0x2EBEF), (0x2F800, 0x2FA1F),
]
# Kana: hiragana, katakana, phonetic extensions, halfwidth katakana, and the
# supplementary-plane kana blocks.
#
# Both katakana runs are split around the middle dot, U+30FB and its halfwidth
# twin U+FF65, which are punctuation that happens to live in a letters block --
# they go in CJK_PUNCT below so 直線・戰略 does not change typeface at the dot.
# The prolonged sound mark U+30FC and the iteration marks are the opposite case:
# they read as part of the word (コーヒー), so they stay with the kana.
KANA = [
    (0x3040, 0x309F), (0x30A0, 0x30FA), (0x30FC, 0x30FF),
    (0x31F0, 0x31FF), (0xFF66, 0xFF9F),
    (0x1B000, 0x1B0FF), (0x1B100, 0x1B12F), (0x1B130, 0x1B16F),
]
# Bopomofo: Taiwanese phonetic annotation, and unambiguously Chinese.
BOPOMOFO = [(0x3100, 0x312F), (0x31A0, 0x31BF)]
# CJK punctuation and the fullwidth/halfwidth forms. Shared by all three faces
# so the brackets in a heading come from the same face as the ideographs.
#
# The two gaps in the fullwidth block are deliberate: U+FF65-FF9F is halfwidth
# katakana and stays pinned to M PLUS U, U+FFA0-FFDC is halfwidth jamo and stays
# pinned to Nanum Gothic. Those are letters that happen to live in a block full
# of punctuation, so they follow the letters rule, not this one.
CJK_PUNCT = [
    (0x3000, 0x303F),  # CJK Symbols and Punctuation -- 、。〈〉「」【】〜
    (0x30FB, 0x30FB),  # Katakana middle dot -- punctuation, carved out of KANA
    (0xFE10, 0xFE1F),  # Vertical Forms
    (0xFE30, 0xFE4F),  # CJK Compatibility Forms -- vertical brackets and dashes
    (0xFE50, 0xFE6F),  # Small Form Variants
    (0xFF01, 0xFF65),  # Fullwidth ASCII, halfwidth punctuation, halfwidth ･
    (0xFFE0, 0xFFEE),  # Fullwidth and halfwidth symbols
]

FONTS = {
    # Korean display face. Hangul is Korean-only, so it is pinned here and no
    # other face may claim it.
    "nanum_gothic.css": HANGUL + HAN + CJK_PUNCT,
    # Japanese display face. Kana is Japanese-only and pinned here.
    "m_plus_u.css": KANA + HAN + CJK_PUNCT,
    # Traditional Chinese display face. Bopomofo is Chinese-only and pinned here;
    # Chinese has no script of its own beyond Han, so this set is the smallest.
    "chiron_goround_tc.css": HAN + BOPOMOFO + CJK_PUNCT,
}

HEADER_MARK = "/* Narrowed by narrow_font_coverage.py"


def parse_range(text):
    out = []
    for part in text.split(","):
        part = part.strip()
        m = re.fullmatch(r"U\+([0-9A-Fa-f]+)-([0-9A-Fa-f]+)", part)
        if m:
            out.append((int(m.group(1), 16), int(m.group(2), 16)))
            continue
        m = re.fullmatch(r"U\+([0-9A-Fa-f]+)", part)
        if m:
            v = int(m.group(1), 16)
            out.append((v, v))
            continue
        raise SystemExit(f"unparsed unicode-range token: {part!r}")
    return out


def intersect(ranges, allowed):
    hits = []
    for a, b in ranges:
        for c, d in allowed:
            lo, hi = max(a, c), min(b, d)
            if lo <= hi:
                hits.append((lo, hi))
    hits.sort()
    merged = []
    for lo, hi in hits:
        # +1 so adjacent runs collapse: U+4e00-4e01, U+4e02 is one range.
        if merged and lo <= merged[-1][1] + 1:
            merged[-1] = (merged[-1][0], max(merged[-1][1], hi))
        else:
            merged.append((lo, hi))
    return merged


def fmt_range(ranges):
    return ", ".join(
        f"U+{lo:x}" if lo == hi else f"U+{lo:x}-{hi:x}" for lo, hi in ranges
    )


def narrow(path, allowed):
    src = Path(path).read_text()
    if src.startswith(HEADER_MARK):
        src = src.split("*/", 1)[1].lstrip("\n")

    # Split into [text, face, text, face, ...] so each face keeps whatever
    # preceded it -- Google's subset comments are formatted differently in each
    # of these files, and some faces have no comment at all.
    parts = re.split(r"(@font-face\s*\{.*?\n\})", src, flags=re.S)
    kept, dropped, before, after = [], 0, 0, 0

    for i in range(1, len(parts), 2):
        preamble, face = parts[i - 1], parts[i]
        found = re.search(r"unicode-range:\s*([^;]+);", face)
        if not found:
            raise SystemExit(f"{path}: @font-face with no unicode-range")
        original = parse_range(found.group(1))
        before += sum(hi - lo + 1 for lo, hi in original)
        narrowed = intersect(original, allowed)
        if not narrowed:
            dropped += 1
            continue
        after += sum(hi - lo + 1 for lo, hi in narrowed)
        face = re.sub(
            r"unicode-range:\s*[^;]+;", f"unicode-range: {fmt_range(narrowed)};", face
        )
        kept.append(preamble + face)

    header = (
        f"{HEADER_MARK} -- this is NOT what Google Fonts serves.\n"
        f"   See that script for why, and re-run it if this file is ever\n"
        f"   re-downloaded. Kept {len(kept)} faces, dropped {dropped}.\n*/\n\n"
    )
    Path(path).write_text(header + "".join(kept).lstrip("\n").rstrip("\n") + "\n")
    print(f"{Path(path).name:26} faces {len(kept) + dropped:>4} -> {len(kept):<4} "
          f"codepoints {before:>6} -> {after}")


if __name__ == "__main__":
    here = Path(__file__).parent
    missing = [f for f in FONTS if not (here / f).exists()]
    if missing:
        raise SystemExit(f"not found (run from app/): {', '.join(missing)}")
    for name, allowed in FONTS.items():
        narrow(here / name, allowed)
