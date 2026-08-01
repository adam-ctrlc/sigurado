"""The shared furniture every form is built from.

One typeface and one size: Arial 12, everywhere, including headings and table
cells. Emphasis is carried by weight and case, never by size. Portrait letter,
no footer, and generous space to write in.

Twelve point is large for a form, so every table here is deliberately narrow:
four or five columns is the most that stays readable across 7.3 inches. Detail
that would need a sixth column is stacked inside a cell instead.
"""

from collections.abc import Sequence
from pathlib import Path

from reportlab.lib import colors
from reportlab.lib.enums import TA_CENTER, TA_LEFT
from reportlab.lib.pagesizes import letter
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib.units import inch
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.platypus import (
    BaseDocTemplate,
    Flowable,
    Frame,
    PageTemplate,
    Paragraph,
    Spacer,
    Table,
    TableStyle,
)

from project import PROJECT, Signatory

# Arial is not one of the fonts built into PDF, so it has to be found on the
# machine. These are where it lives on Windows, macOS, and a Linux box with the
# Microsoft fonts installed.
_ARIAL_CANDIDATES: tuple[tuple[str, tuple[str, ...]], ...] = (
    (
        "Arial",
        (
            r"C:\Windows\Fonts\arial.ttf",
            "/Library/Fonts/Arial.ttf",
            "/usr/share/fonts/truetype/msttcorefonts/Arial.ttf",
        ),
    ),
    (
        "Arial-Bold",
        (
            r"C:\Windows\Fonts\arialbd.ttf",
            "/Library/Fonts/Arial Bold.ttf",
            "/usr/share/fonts/truetype/msttcorefonts/Arial_Bold.ttf",
        ),
    ),
)


def _register_arial() -> tuple[str, str]:
    """Registers Arial, or falls back to the metrically identical Helvetica."""
    found: list[str] = []
    for name, paths in _ARIAL_CANDIDATES:
        for path in paths:
            if Path(path).exists():
                pdfmetrics.registerFont(TTFont(name, path))
                found.append(name)
                break

    if len(found) == 2:
        pdfmetrics.registerFontFamily("Arial", normal="Arial", bold="Arial-Bold")
        return "Arial", "Arial-Bold"

    print("Arial was not found on this machine; using Helvetica, which shares its metrics.")
    return "Helvetica", "Helvetica-Bold"


FONT, FONT_BOLD = _register_arial()

SIZE = 12
"""Every piece of text on every form. Nothing is smaller and nothing is larger."""

LEADING = 15.5

INK = colors.HexColor("#000000")
RULE = colors.HexColor("#555555")
HAIRLINE = colors.HexColor("#999999")
BAND = colors.HexColor("#E8E8E8")

MARGIN = 0.6 * inch

TITLE = ParagraphStyle(
    "title",
    fontName=FONT_BOLD,
    fontSize=SIZE,
    leading=LEADING,
    alignment=TA_CENTER,
    textColor=INK,
)
SUBTITLE = ParagraphStyle(
    "subtitle",
    fontName=FONT,
    fontSize=SIZE,
    leading=LEADING,
    alignment=TA_CENTER,
    textColor=INK,
)
HEADING = ParagraphStyle(
    "heading",
    fontName=FONT_BOLD,
    fontSize=SIZE,
    leading=LEADING,
    spaceBefore=10,
    spaceAfter=5,
    textColor=INK,
    # Safe for prose and short blocks. Tables carry their own title row instead,
    # because keeping a heading with a table long enough to split pushes the
    # whole table to the next page and leaves the current one half empty.
    keepWithNext=1,
)
BODY = ParagraphStyle(
    "body",
    fontName=FONT,
    fontSize=SIZE,
    leading=LEADING,
    alignment=TA_LEFT,
    textColor=INK,
)
CELL = ParagraphStyle("cell", parent=BODY, leading=14.5)
CELL_BOLD = ParagraphStyle("cellBold", parent=CELL, fontName=FONT_BOLD)
NOTE = ParagraphStyle("note", parent=BODY)

WRITING_ROW = 30
"""Height of a row somebody has to write in. A row sized to its text is fine to
read and impossible to fill in by hand."""


class Rule(Flowable):
    """A plain horizontal line, for signing on or separating blocks."""

    def __init__(self, width: float, *, thickness: float = 0.7, color=RULE) -> None:
        super().__init__()
        self.width = width
        self.height = thickness
        self._thickness = thickness
        self._color = color

    def draw(self) -> None:
        self.canv.setStrokeColor(self._color)
        self.canv.setLineWidth(self._thickness)
        self.canv.line(0, 0, self.width, 0)


class FormDoc(BaseDocTemplate):
    """A portrait letter page with nothing on it but the form.

    The frame carries no padding of its own. A default frame inset the content
    by six points on each side, which left a full-width table wider than the
    space it was given, and reportlab centred the overflow: every table sat six
    points left of every paragraph. With no padding, `doc.width` is exactly the
    usable width and the two line up.
    """

    def __init__(self, path: Path, *, form_name: str) -> None:
        super().__init__(
            str(path),
            pagesize=letter,
            leftMargin=MARGIN,
            rightMargin=MARGIN,
            topMargin=MARGIN,
            bottomMargin=MARGIN,
            title=form_name,
            author=PROJECT.short_name,
            subject=PROJECT.title,
        )
        self.form_name = form_name
        frame = Frame(
            self.leftMargin,
            self.bottomMargin,
            self.width,
            self.height,
            id="body",
            leftPadding=0,
            rightPadding=0,
            topPadding=0,
            bottomPadding=0,
        )
        self.addPageTemplates(PageTemplate(id="form", frames=[frame]))


def title_block(form_name: str, purpose: str) -> list[Flowable]:
    """The heading every form opens with."""
    out: list[Flowable] = []
    if PROJECT.school:
        out.append(Paragraph(f"<b>{PROJECT.school.upper()}</b>", SUBTITLE))
    if PROJECT.department:
        out.append(Paragraph(PROJECT.department, SUBTITLE))
    if PROJECT.school or PROJECT.department:
        out.append(Spacer(1, 8))

    out.append(Paragraph(f"<b>{form_name.upper()}</b>", TITLE))
    out.append(Spacer(1, 6))
    out.append(Paragraph(PROJECT.title, SUBTITLE))
    out.append(Spacer(1, 12))
    out.append(Paragraph(purpose, BODY))
    out.append(Spacer(1, 14))
    return out


def field_rows(fields: Sequence[tuple[str, str]], width: float) -> Table:
    """Labelled fields, one per row, each with a line to write on.

    One per row rather than two side by side: at twelve point a pair of columns
    leaves neither of them long enough to write a name in.
    """
    label_width = width * 0.30
    rows: list[list[Flowable]] = [
        [Paragraph(f"<b>{label}</b>", CELL), Paragraph(value, CELL)]
        for label, value in fields
    ]

    table = Table(rows, colWidths=[label_width, width - label_width])
    table.setStyle(
        TableStyle(
            [
                ("VALIGN", (0, 0), (-1, -1), "BOTTOM"),
                ("LEFTPADDING", (0, 0), (-1, -1), 0),
                ("RIGHTPADDING", (0, 0), (-1, -1), 0),
                ("TOPPADDING", (0, 0), (-1, -1), 9),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 3),
                ("LINEBELOW", (1, 0), (1, -1), 0.7, RULE),
            ]
        )
    )
    return table


def identity_block(width: float, extra: Sequence[tuple[str, str]] = ()) -> list[Flowable]:
    """Who the project is, one field per row, then the members one per row."""
    fields: list[tuple[str, str]] = [
        ("Subject", PROJECT.subject),
        ("Section", PROJECT.section),
        ("School year", PROJECT.school_year),
        ("Adviser", PROJECT.adviser),
        ("Date prepared", ""),
    ]
    fields.extend(extra)

    out: list[Flowable] = [field_rows(fields, width), Spacer(1, 14)]
    out.append(Paragraph("<b>Prepared by</b>", BODY))
    out.append(
        field_rows(
            [(f"{i}.", name) for i, name in enumerate(PROJECT.member_lines(), start=1)],
            width,
        )
    )
    return out


def heading(text: str) -> Paragraph:
    return Paragraph(f"<b>{text}</b>", HEADING)


def note(text: str) -> Paragraph:
    return Paragraph(text, NOTE)


def body(text: str) -> Paragraph:
    return Paragraph(text, BODY)


def data_table(
    header: Sequence[str],
    rows: Sequence[Sequence[str]],
    widths: Sequence[float],
    *,
    title: str = "",
    blank_rows: int = 0,
    align_right: Sequence[int] = (),
    show_header: bool = True,
) -> Table:
    """A ruled table. Rows that are entirely empty get writing height.

    `title` becomes a spanning row at the top. Carrying the title inside the
    table means it can never be left stranded at the foot of a page, and it
    repeats with the column headers when the table runs over.
    """
    body_rows: list[list[Flowable]] = []
    heights: list[float | None] = []
    lead = 0

    if title:
        body_rows.append([Paragraph(f"<b>{title}</b>", CELL)] + [""] * (len(header) - 1))
        heights.append(None)
        lead = 1

    if show_header:
        body_rows.append([Paragraph(f"<b>{h}</b>", CELL) for h in header])
        heights.append(None)

    for row in rows:
        body_rows.append([Paragraph(str(cell), CELL) for cell in row])
        blank = all(str(cell).strip() == "" for cell in row)
        heights.append(WRITING_ROW if blank else None)

    for _ in range(blank_rows):
        body_rows.append([Paragraph("", CELL) for _ in header])
        heights.append(WRITING_ROW)

    table = Table(
        body_rows,
        colWidths=list(widths),
        rowHeights=heights,
        repeatRows=lead + (1 if show_header else 0),
    )
    style = [
        ("GRID", (0, 0), (-1, -1), 0.6, HAIRLINE),
        ("BOX", (0, 0), (-1, -1), 1.0, RULE),
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("TOPPADDING", (0, 0), (-1, -1), 7),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 7),
        ("LEFTPADDING", (0, 0), (-1, -1), 7),
        ("RIGHTPADDING", (0, 0), (-1, -1), 7),
    ]
    if title:
        style.append(("SPAN", (0, 0), (-1, 0)))
        style.append(("BACKGROUND", (0, 0), (-1, 0), BAND))
    if show_header:
        style.append(("BACKGROUND", (0, lead), (-1, lead), BAND))
        style.append(("LINEBELOW", (0, lead), (-1, lead), 1.0, RULE))
    for column in align_right:
        style.append(("ALIGN", (column, lead), (column, -1), "RIGHT"))
    table.setStyle(TableStyle(style))
    return table


def signature_block(
    signatories: Sequence[Signatory],
    width: float,
    *,
    caption: str = "",
) -> list[Flowable]:
    """One signatory per row: the role, a rule to sign on, then the date line.

    Stacked rather than side by side, so each rule runs most of the page width
    and there is room to sign at twelve point.
    """
    if not signatories:
        return []

    out: list[Flowable] = []
    if caption:
        out.append(Paragraph(caption, BODY))
        out.append(Spacer(1, 14))

    rule_width = width * 0.60
    date_width = width * 0.32
    gap = width - rule_width - date_width

    for person in signatories:
        out.append(Paragraph(f"<b>{person.role}</b>", BODY))
        out.append(Spacer(1, 28))

        rules = Table(
            [[Rule(rule_width), "", Rule(date_width)]],
            colWidths=[rule_width, gap, date_width],
        )
        rules.setStyle(
            TableStyle(
                [
                    ("VALIGN", (0, 0), (-1, -1), "BOTTOM"),
                    ("LEFTPADDING", (0, 0), (-1, -1), 0),
                    ("RIGHTPADDING", (0, 0), (-1, -1), 0),
                    ("TOPPADDING", (0, 0), (-1, -1), 0),
                    ("BOTTOMPADDING", (0, 0), (-1, -1), 0),
                ]
            )
        )
        out.append(rules)

        under = Table(
            [
                [
                    Paragraph(person.name or "Signature over printed name", CELL),
                    "",
                    Paragraph("Date signed", CELL),
                ]
            ],
            colWidths=[rule_width, gap, date_width],
        )
        under.setStyle(
            TableStyle(
                [
                    ("VALIGN", (0, 0), (-1, -1), "TOP"),
                    ("LEFTPADDING", (0, 0), (-1, -1), 0),
                    ("RIGHTPADDING", (0, 0), (-1, -1), 0),
                    ("TOPPADDING", (0, 0), (-1, -1), 3),
                    ("BOTTOMPADDING", (0, 0), (-1, -1), 0),
                ]
            )
        )
        out.append(under)

        if person.subtitle:
            out.append(Paragraph(person.subtitle, CELL))
        out.append(Spacer(1, 20))

    return out


def remarks_block(width: float, *, lines: int = 4, label: str = "Remarks") -> list[Flowable]:
    """Ruled space for whatever the form did not anticipate."""
    out: list[Flowable] = [heading(label)]
    for _ in range(lines):
        out.append(Spacer(1, 24))
        out.append(Rule(width, thickness=0.6, color=HAIRLINE))
    out.append(Spacer(1, 10))
    return out


def build(doc: FormDoc, story: Sequence[Flowable]) -> Path:
    """Writes the PDF and returns where it landed."""
    doc.build(list(story))
    return Path(doc.filename)


def output_dir() -> Path:
    """Generated forms live beside the scripts, in `out`."""
    directory = Path(__file__).parent / "out"
    directory.mkdir(exist_ok=True)
    return directory
