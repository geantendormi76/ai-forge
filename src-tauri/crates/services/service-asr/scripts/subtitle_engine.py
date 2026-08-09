from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Callable, Iterable, Iterator


# ==============================================================================
# 1. 核心数据结构 (Models) - 注入“紫电青霜”专属配色调色板
# ==============================================================================

@dataclass(slots=True)
class SubtitleSegment:
    id: str
    start: float
    end: float
    speaker: str
    text: str

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, data: dict[str, Any], *, fallback_id: str | None = None) -> SubtitleSegment:
        return cls(
            id=str(data.get("id") or fallback_id or ""),
            start=float(data.get("start", 0.0)),
            end=float(data.get("end", 0.0)),
            speaker=str(data.get("speaker") or "S01"),
            text=str(data.get("text") or ""),
        )


# ⚡ 紫电青霜 ASS 字幕调色板 (BGR 格式)
SPEAKER_COLORS = [
    "&H00F8B4D8",  # 💜 主色-淡紫 #D8B4F8 (紫电 - S01)
    "&H00FCF3A5",  # 🩵 副色-淡青 #A5F3FC (青霜 - S02)
    "&H00FFB56B",  # 橙粉 #FFB56B (S03)
    "&H008FF286",  # 荧光绿 #86F28F (S04)
    "&H0000D7FF",  # 金黄 #FFD700 (S05)
    "&H00DB8EFF",  # 玫粉 #FF8EDB (S06)
    "&H00FFE75B",  # 天蓝 #5BE7FF (S07)
    "&H00D8D8D8",  # 浅灰 #D8D8D8 (S08)
]


# ==============================================================================
# 2. 零正则状态机解析器 (TranscriptStreamParser)
# ==============================================================================

class TranscriptStreamParser:
    """按字符扫描解析 [start][Sxx]text[end] 格式，拒绝正则崩溃"""

    _SEEK_START = 0
    _READ_START = 1
    _EXPECT_SPEAKER_OPEN = 2
    _READ_SPEAKER = 3
    _READ_TEXT = 4
    _READ_END = 5
    _AFTER_END = 6

    def __init__(self, *, strip_text: bool = True, skip_empty: bool = True):
        self.strip_text = strip_text
        self.skip_empty = skip_empty
        self._state = self._SEEK_START
        self._token: list[str] = []
        self._text: list[str] = []
        self._pending_after_end: list[str] = []
        self._start: float | None = None
        self._end: float | None = None
        self._end_token = ""
        self._speaker: str | None = None

    def reset(self) -> None:
        self._state = self._SEEK_START
        self._token.clear()
        self._text.clear()
        self._pending_after_end.clear()
        self._start = None
        self._end = None
        self._end_token = ""
        self._speaker = None

    def feed(self, chunk: str) -> list[SubtitleSegment]:
        segments: list[SubtitleSegment] = []
        self.feed_into(chunk, segments.append)
        return segments

    def feed_into(self, chunk: str, emit: Callable[[SubtitleSegment], None]) -> None:
        for ch in chunk:
            state = self._state
            if state == self._SEEK_START:
                self._seek_start(ch)
            elif state == self._READ_START:
                self._read_start(ch)
            elif state == self._EXPECT_SPEAKER_OPEN:
                self._expect_speaker_open(ch)
            elif state == self._READ_SPEAKER:
                self._read_speaker(ch)
            elif state == self._READ_TEXT:
                self._read_text(ch)
            elif state == self._READ_END:
                self._read_end(ch, emit)
            elif state == self._AFTER_END:
                self._after_end(ch, emit)

    def close(self) -> list[SubtitleSegment]:
        segments: list[SubtitleSegment] = []
        self.close_into(segments.append)
        return segments

    def close_into(self, emit: Callable[[SubtitleSegment], None]) -> None:
        if self._state == self._AFTER_END:
            self._emit_segment(emit)
        self.reset()

    def _seek_start(self, ch: str) -> None:
        if ch == "[":
            self._token.clear()
            self._state = self._READ_START

    def _read_start(self, ch: str) -> None:
        if ch == "]":
            start = _parse_timestamp(self._token)
            if start is None:
                self.reset()
                return
            self._start = start
            self._state = self._EXPECT_SPEAKER_OPEN
            self._token.clear()
            return

        if _is_timestamp_char(ch):
            self._token.append(ch)
            if len(self._token) <= 32:
                return
        self.reset()
        if ch == "[":
            self._state = self._READ_START

    def _expect_speaker_open(self, ch: str) -> None:
        if ch == "[":
            self._token.clear()
            self._state = self._READ_SPEAKER
        elif not ch.isspace():
            self.reset()

    def _read_speaker(self, ch: str) -> None:
        if ch == "]":
            speaker = _parse_speaker(self._token)
            if speaker is None:
                self.reset()
                return
            self._speaker = speaker
            self._text.clear()
            self._state = self._READ_TEXT
            self._token.clear()
            return

        if _is_speaker_char(ch):
            self._token.append(ch)
            if len(self._token) <= 16:
                return
        self.reset()
        if ch == "[":
            self._state = self._READ_START

    def _read_text(self, ch: str) -> None:
        if ch == "[":
            self._token.clear()
            self._state = self._READ_END
        else:
            self._text.append(ch)

    def _read_end(self, ch: str, emit: Callable[[SubtitleSegment], None]) -> None:
        if ch == "]":
            end = _parse_timestamp(self._token)
            if end is not None and self._start is not None and end >= self._start:
                self._end = end
                self._end_token = "".join(self._token)
                self._pending_after_end.clear()
                self._state = self._AFTER_END
            else:
                self._text.append("[")
                self._text.extend(self._token)
                self._text.append("]")
                self._state = self._READ_TEXT
            self._token.clear()
            return

        if _is_timestamp_char(ch):
            self._token.append(ch)
            if len(self._token) <= 32:
                return

        self._text.append("[")
        self._text.extend(self._token)
        self._text.append(ch)
        self._token.clear()
        self._state = self._READ_TEXT

    def _after_end(self, ch: str, emit: Callable[[SubtitleSegment], None]) -> None:
        if ch == "[":
            self._emit_segment(emit)
            self._token.clear()
            self._state = self._READ_START
            return

        if ch.isspace():
            self._pending_after_end.append(ch)
            return

        self._text.append("[")
        self._text.append(self._end_token)
        self._text.append("]")
        self._text.extend(self._pending_after_end)
        self._text.append(ch)
        self._pending_after_end.clear()
        self._end = None
        self._end_token = ""
        self._state = self._READ_TEXT

    def _emit_segment(self, emit: Callable[[SubtitleSegment], None]) -> None:
        if self._start is None or self._end is None or self._speaker is None:
            self.reset()
            return

        text = "".join(self._text)
        if self.strip_text:
            text = text.strip()
        if text or not self.skip_empty:
            emit(
                SubtitleSegment(
                    id="",
                    start=self._start,
                    end=self._end,
                    speaker=self._speaker,
                    text=text,
                )
            )

        self.reset()


def parse_transcript(text: str) -> list[SubtitleSegment]:
    parser = TranscriptStreamParser()
    segments = parser.feed(text)
    segments.extend(parser.close())
    for idx, seg in enumerate(segments, start=1):
        seg.id = f"seg_{idx:04d}"
    return segments


def _parse_timestamp(chars: list[str]) -> float | None:
    if not chars:
        return None
    dot_count = 0
    digit_count = 0
    for ch in chars:
        if "0" <= ch <= "9":
            digit_count += 1
        elif ch == ".":
            dot_count += 1
            if dot_count > 1:
                return None
        else:
            return None
    if digit_count == 0:
        return None
    return float("".join(chars))


def _parse_speaker(chars: list[str]) -> str | None:
    if len(chars) < 2 or chars[0] != "S":
        return None
    for ch in chars[1:]:
        if not ("0" <= ch <= "9"):
            return None
    return "".join(chars)


def _is_timestamp_char(ch: str) -> bool:
    return ("0" <= ch <= "9") or ch == "."


def _is_speaker_char(ch: str) -> bool:
    return ch == "S" or ("0" <= ch <= "9")


# ==============================================================================
# 3. 字幕工业级归一化 (Normalize & Postprocess)
# ==============================================================================

DEFAULT_MIN_DURATION = 1.0
DEFAULT_MAX_DURATION = 6.0
DEFAULT_MAX_CHARS = 24
DEFAULT_MERGE_GAP = 0.3
PUNCTUATION = "。！？!?；;，,、 "


def normalize_segments(
    segments: Iterable[SubtitleSegment | dict],
    *,
    min_duration: float = DEFAULT_MIN_DURATION,
    max_duration: float = DEFAULT_MAX_DURATION,
    max_chars: int = DEFAULT_MAX_CHARS,
    merge_gap: float = DEFAULT_MERGE_GAP,
    regenerate_ids: bool = True,
) -> list[SubtitleSegment]:
    prepared = _prepare_segments(segments)
    prepared = _fix_overlaps(prepared, min_duration=min_duration)
    prepared = _merge_adjacent(prepared, merge_gap=merge_gap, max_chars=max_chars)
    prepared = _split_long_segments(
        prepared,
        min_duration=min_duration,
        max_duration=max_duration,
        max_chars=max_chars,
    )
    prepared = _fix_overlaps(prepared, min_duration=min_duration)
    if regenerate_ids:
        for index, segment in enumerate(prepared, start=1):
            segment.id = f"seg_{index:04d}"
    return prepared


def _prepare_segments(segments: Iterable[SubtitleSegment | dict]) -> list[SubtitleSegment]:
    prepared: list[SubtitleSegment] = []
    for index, item in enumerate(segments, start=1):
        segment = item if isinstance(item, SubtitleSegment) else SubtitleSegment.from_dict(item, fallback_id=f"seg_{index:04d}")
        text = segment.text.strip()
        if not text:
            continue
        start = max(0.0, float(segment.start))
        end = max(start, float(segment.end))
        prepared.append(
            SubtitleSegment(
                id=segment.id or f"seg_{index:04d}",
                start=start,
                end=end,
                speaker=segment.speaker or "S01",
                text=text,
            )
        )
    prepared.sort(key=lambda segment: (segment.start, segment.end))
    return prepared


def _fix_overlaps(segments: list[SubtitleSegment], *, min_duration: float) -> list[SubtitleSegment]:
    cursor = 0.0
    fixed: list[SubtitleSegment] = []
    for segment in segments:
        start = max(segment.start, cursor)
        end = max(segment.end, start + min_duration)
        fixed.append(
            SubtitleSegment(
                id=segment.id,
                start=start,
                end=end,
                speaker=segment.speaker,
                text=segment.text,
            )
        )
        cursor = end
    return fixed


def _merge_adjacent(segments: list[SubtitleSegment], *, merge_gap: float, max_chars: int) -> list[SubtitleSegment]:
    if not segments:
        return []

    merged = [segments[0]]
    for segment in segments[1:]:
        previous = merged[-1]
        gap = segment.start - previous.end
        combined_text = _join_text(previous.text, segment.text)
        can_merge = (
            previous.speaker == segment.speaker
            and 0 <= gap <= merge_gap
            and len(combined_text) <= max_chars * 2
        )
        if can_merge:
            merged[-1] = SubtitleSegment(
                id=previous.id,
                start=previous.start,
                end=max(previous.end, segment.end),
                speaker=previous.speaker,
                text=combined_text,
            )
        else:
            merged.append(segment)
    return merged


def _split_long_segments(
    segments: list[SubtitleSegment],
    *,
    min_duration: float,
    max_duration: float,
    max_chars: int,
) -> list[SubtitleSegment]:
    output: list[SubtitleSegment] = []
    for segment in segments:
        duration = segment.end - segment.start
        if duration <= max_duration and len(segment.text) <= max_chars:
            output.append(segment)
            continue

        chunks = _split_text(segment.text, max_chars=max_chars)
        if len(chunks) <= 1:
            output.append(segment)
            continue

        total_chars = sum(max(len(chunk), 1) for chunk in chunks)
        cursor = segment.start
        for index, chunk in enumerate(chunks):
            if index == len(chunks) - 1:
                end = segment.end
            else:
                ratio = max(len(chunk), 1) / total_chars
                end = cursor + max(min_duration, duration * ratio)
                end = min(end, segment.end - min_duration * (len(chunks) - index - 1))
            output.append(
                SubtitleSegment(
                    id=f"{segment.id}_{index + 1}",
                    start=cursor,
                    end=max(end, cursor + min_duration),
                    speaker=segment.speaker,
                    text=chunk,
                )
            )
            cursor = output[-1].end
    return output


def _split_text(text: str, *, max_chars: int) -> list[str]:
    text = text.strip()
    if len(text) <= max_chars:
        return [text]

    chunks: list[str] = []
    current: list[str] = []
    for ch in text:
        current.append(ch)
        should_cut = len(current) >= max_chars or (ch in PUNCTUATION and len(current) >= max_chars // 2)
        if should_cut:
            chunks.append("".join(current).strip())
            current.clear()
    if current:
        chunks.append("".join(current).strip())

    compact: list[str] = []
    for chunk in chunks:
        if not chunk:
            continue
        if compact and len(compact[-1]) + len(chunk) <= max_chars:
            compact[-1] = _join_text(compact[-1], chunk)
        else:
            compact.append(chunk)
    return compact


def _join_text(left: str, right: str) -> str:
    if not left:
        return right
    if not right:
        return left
    if left[-1].isascii() and right[0].isascii():
        return f"{left} {right}"
    return f"{left}{right}"


# ==============================================================================
# 4. SRT / ASS 多格式渲染器 (Exporters)
# ==============================================================================

def format_srt_time(seconds: float) -> str:
    milliseconds = max(0, round(float(seconds) * 1000))
    hours, remainder = divmod(milliseconds, 3_600_000)
    minutes, remainder = divmod(remainder, 60_000)
    secs, millis = divmod(remainder, 1000)
    return f"{hours:02d}:{minutes:02d}:{secs:02d},{millis:03d}"


def format_ass_time(seconds: float) -> str:
    centiseconds = max(0, round(float(seconds) * 100))
    hours, remainder = divmod(centiseconds, 360_000)
    minutes, remainder = divmod(remainder, 6_000)
    secs, centis = divmod(remainder, 100)
    return f"{hours:d}:{minutes:02d}:{secs:02d}.{centis:02d}"


def export_json(segments: Iterable[SubtitleSegment], *, indent: int = 2) -> str:
    return json.dumps([segment.to_dict() for segment in segments], ensure_ascii=False, indent=indent) + "\n"


def export_srt(
    segments: Iterable[SubtitleSegment],
    *,
    show_speaker: bool = True,
    speaker_names: dict[str, str] | None = None,
) -> str:
    blocks = []
    for index, segment in enumerate(segments, start=1):
        spk_prefix = f"[{segment.speaker}] " if show_speaker and segment.speaker else ""
        if speaker_names and segment.speaker in speaker_names:
            spk_prefix = f"[{speaker_names[segment.speaker]}] "
        text = f"{spk_prefix}{segment.text}"
        blocks.append(
            "\n".join(
                [
                    str(index),
                    f"{format_srt_time(segment.start)} --> {format_srt_time(segment.end)}",
                    text,
                ]
            )
        )
    return "\n\n".join(blocks) + ("\n" if blocks else "")


def export_ass(
    segments: Iterable[SubtitleSegment],
    *,
    video_width: int = 1920,
    video_height: int = 1080,
    show_speaker: bool = True,
) -> str:
    """生成包含紫电青霜 Speaker 彩色字幕与防重叠格式的 ASS 文件"""
    font_size = max(28, round(video_height * 0.045))
    segments_list = list(segments)
    speakers = sorted({segment.speaker for segment in segments_list})

    style_lines = [
        f"Style: Default,Noto Sans CJK SC,{font_size},&H00F8B4D8,&H000000FF,&H00000000,&H64000000,0,0,0,0,100,100,0,0,1,3,1,2,48,48,56,1"
    ]

    for index, speaker in enumerate(speakers):
        color = SPEAKER_COLORS[index % len(SPEAKER_COLORS)]
        spk_name = f"Speaker_{speaker}"
        style_lines.append(
            f"Style: {spk_name},Noto Sans CJK SC,{font_size},{color},&H000000FF,&H00000000,&H64000000,0,0,0,0,100,100,0,0,1,3,1,2,48,48,56,1"
        )

    dialogue_lines = []
    for segment in segments_list:
        style_name = f"Speaker_{segment.speaker}" if segment.speaker in speakers else "Default"
        spk_prefix = f"[{segment.speaker}] " if show_speaker and segment.speaker else ""
        text = f"{spk_prefix}{segment.text}".replace("\\", "\\\\").replace("{", "(").replace("}", ")").replace("\n", "\\N")
        dialogue_lines.append(
            f"Dialogue: 0,{format_ass_time(segment.start)},{format_ass_time(segment.end)},"
            f"{style_name},,0,0,56,,{text}"
        )

    return "\n".join(
        [
            "[Script Info]",
            "ScriptType: v4.00+",
            "WrapStyle: 2",
            "ScaledBorderAndShadow: yes",
            f"PlayResX: {video_width}",
            f"PlayResY: {video_height}",
            "",
            "[V4+ Styles]",
            "Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, "
            "Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, "
            "Shadow, Alignment, MarginL, MarginR, MarginV, Encoding",
            *style_lines,
            "",
            "[Events]",
            "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text",
            *dialogue_lines,
            "",
        ]
    )
