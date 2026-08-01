"""The shared furniture every form is built from.

Short bond, conservative margins, one typeface, and a footer that says which
script produced the page. Forms differ in their tables, not in their look.
"""

from collections.abc import Sequence
from pathlib import Path
from typing import Literal

from reportlab.lib import colors
from reportlab.lib.enums import TA_CENTER, TA_LEFT
from reportlab.lib.pagesizes import letter, landscape
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib.units import inch
from reportlab.pdfgen.canvas import Canvas
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

FONT = "Helvetica"
FONT_BOLD = "Helvetica-Bold"

INK = colors.HexColor("#111111")
RULE = colors.HexColor("#666666")
HAIRLINE = colors.HexColor("#BBBBBB")
BAND = colors.HexColor("#EFEFEF")

MARGIN = 0.55 * inch
Orientation = Literal["portrait", "landscape"]

TITLE = ParagraphStyle(
    "title",
    fontName=FONT_BOLD,
    fontSize=12.5,
    leading=15,
    alignment=TA_CENTER,
    textColor=INK,
)
SUBTITLE = ParagraphStyle(
    "subtitle",
    fontName=FONT,
    fontSize=9,
    leading=12,
    alignment=TA_CENTER,
    textColor=INK,
)
HEADING = ParagraphStyle(
    "heading",
    fontName=FONT_BOLD,
    fontSize=9.5,
    leading=12,
    spaceBefore=8,
    spaceAfter=3,
    textColor=INK,
)
BODY = ParagraphStyle(
    "body",
    fontName=FONT,
    fontSize=8.5,
    leading=11,
    alignment=TA_LEFT,
    textColor=INK,
)
CELL = ParagraphStyle("cell", parent=BODY, fontSize=8, leading=10)
CELL_BOLD = ParagraphStyle("cellBold", parent=CELL, fontName=FONT_BOLD)
NOTE = ParagraphStyle("note", parent=BODY, fontSize=7.5, leading=10, textColor=RULE)


class Rule(Flowable):
    """A plain horizontal line, for signing on or separating blocks."""

    def __init__(self, width: float, *, thickness: float = 0.6, color=RULE) -> None:
        super().__init__()
        self.width = width
        self.height = thickness
        self._thickness = thickness
        self._color = color

    def draw(self) -> None:
        self.canv.setStrokeColor(self._color)
        self.canv.setLineWidth(self._thickness)
        self.canv.line(0, 0, self.width, 0)


def _footer(canvas: Canvas, doc: BaseDocTemplate) -> None:
    """Page furniture: which form, which page, and where it came from."""
    canvas.saveState()
    canvas.setFont(FONT, 7)
    canvas.setFillColor(RULE)

    left = doc.leftMargin
    right = doc.pagesize[0] - doc.rightMargin
    y = 0.38 * inch

    canvas.setStrokeColor(HAIRLINE)
    canvas.setLineWidth(0.5)
    canvas.line(left, y + 10, right, y + 10)

    canvas.drawString(left, y, f"{PROJECT.short_name} | {doc.form_name}")
    canvas.drawRightString(right, y, f"Page {canvas.getPageNumber()}")
    canvas.drawCentredString(
        (left + right) / 2, y, "Generated, then completed and signed by hand"
    )
    canvas.restoreState()


class FormDoc(BaseDocTemplate):
    """A short bond page with a footer naming the form."""

    def __init__(
        self,
        path: Path,
        *,
        form_name: str,
        orientation: Orientation = "portrait",
    ) -> None:
        size = landscape(letter) if orientation == "landscape" else letter
        super().__init__(
            str(path),
            pagesize=size,
            leftMargin=MARGIN,
            rightMargin=MARGIN,
            topMargin=MARGIN,
            bottomMargin=0.7 * inch,
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
        self.addPageTemplates(PageTemplate(id="form", frames=[frame], onPage=_footer))


def title_block(form_name: str, purpose: str) -> list[Flowable]:
    """The heading every form opens with."""
    out: list[Flowable] = []
    if PROJECT.school:
        out.append(Paragraph(PROJECT.school.upper(), SUBTITLE))
    if PROJECT.department:
        out.append(Paragraph(PROJECT.department, SUBTITLE))
    out.append(Spacer(1, 6))
    out.append(Paragraph(form_name.upper(), TITLE))
    out.append(Spacer(1, 3))
    out.append(Paragraph(PROJECT.title, SUBTITLE))
    out.append(Spacer(1, 5))
    out.append(Paragraph(purpose, NOTE))
    out.append(Spacer(1, 9))
    return out


def _field(label: str, value: str, width: float) -> Table:
    """A label with either a value or a line to write one on."""
    text = value if value else ""
    inner = Table(
        [[Paragraph(f"<b>{label}</b>", CELL), Paragraph(text, CELL)]],
        colWidths=[width * 0.32, width * 0.68],
    )
    inner.setStyle(
        TableStyle(
            [
                ("VALIGN", (0, 0), (-1, -1), "BOTTOM"),
                ("LEFTPADDING", (0, 0), (-1, -1), 0),
                ("RIGHTPADDING", (0, 0), (-1, -1), 4),
                ("TOPPADDING", (0, 0), (-1, -1), 3),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 3),
                ("LINEBELOW", (1, 0), (1, 0), 0.5, RULE),
            ]
        )
    )
    return inner


def identity_block(width: float, extra: Sequence[tuple[str, str]] = ()) -> Flowable:
    """Who the project is, in two columns, with blanks where they belong."""
    left: list[tuple[str, str]] = [
        ("Subject", PROJECT.subject),
        ("Section", PROJECT.section),
        ("School year", PROJECT.school_year),
    ]
    right: list[tuple[str, str]] = [
        ("Adviser", PROJECT.adviser),
        ("Date prepared", ""),
        ("Document no.", ""),
    ]
    right.extend(extra)

    half = width / 2 - 6
    rows: list[list[Flowable]] = []
    for i in range(max(len(left), len(right))):
        cells: list[Flowable] = []
        for column in (left, right):
            if i < len(column):
                label, value = column[i]
                cells.append(_field(label, value, half))
            else:
                cells.append(Spacer(1, 1))
        rows.append(cells)

    members = ", ".join(PROJECT.member_lines())
    rows.append([_field("Prepared by", members, width), Spacer(1, 1)])

    table = Table(rows, colWidths=[half, half])
    table.setStyle(
        TableStyle(
            [
                ("VALIGN", (0, 0), (-1, -1), "TOP"),
                ("LEFTPADDING", (0, 0), (-1, -1), 0),
                ("RIGHTPADDING", (0, 0), (-1, -1), 0),
                ("TOPPADDING", (0, 0), (-1, -1), 1),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 1),
                ("SPAN", (0, len(rows) - 1), (1, len(rows) - 1)),
            ]
        )
    )
    return table


def heading(text: str) -> Paragraph:
    return Paragraph(text, HEADING)


def note(text: str) -> Paragraph:
    return Paragraph(text, NOTE)


WRITING_ROW = 24
"""Height of a row somebody has to write in, in points. A default-height row is
fine to read and impossible to write in."""


def data_table(
    header: Sequence[str],
    rows: Sequence[Sequence[str]],
    widths: Sequence[float],
    *,
    blank_rows: int = 0,
    align_right: Sequence[int] = (),
    show_header: bool = True,
) -> Table:
    """A ruled table, with empty rows at the bottom for writing more in.

    Rows that are entirely empty are given writing height, so a form does not
    print a row too thin to fill in.
    """
    body: list[list[Flowable]] = []
    heights: list[float | None] = []

    if show_header:
        body.append([Paragraph(h, CELL_BOLD) for h in header])
        heights.append(None)

    for row in rows:
        body.append([Paragraph(str(cell), CELL) for cell in row])
        blank = all(str(cell).strip() == "" for cell in row)
        heights.append(WRITING_ROW if blank else None)

    for _ in range(blank_rows):
        body.append([Paragraph("", CELL) for _ in header])
        heights.append(WRITING_ROW)

    table = Table(
        body,
        colWidths=list(widths),
        rowHeights=heights,
        repeatRows=1 if show_header else 0,
    )
    style = [
        ("GRID", (0, 0), (-1, -1), 0.5, HAIRLINE),
        ("BOX", (0, 0), (-1, -1), 0.8, RULE),
        ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
        ("TOPPADDING", (0, 0), (-1, -1), 4),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 4),
        ("LEFTPADDING", (0, 0), (-1, -1), 5),
        ("RIGHTPADDING", (0, 0), (-1, -1), 5),
    ]
    if show_header:
        style.append(("BACKGROUND", (0, 0), (-1, 0), BAND))
        style.append(("LINEBELOW", (0, 0), (-1, 0), 0.8, RULE))
    for column in align_right:
        style.append(("ALIGN", (column, 0), (column, -1), "RIGHT"))
    table.setStyle(TableStyle(style))
    return table


def signature_block(
    signatories: Sequence[Signatory],
    width: float,
    *,
    columns: int = 2,
    caption: str = "",
) -> list[Flowable]:
    """Names on rules, in a grid, kept on one page with their heading.

    Two columns by default: any more and the rules get too short to sign on.
    """
    if not signatories:
        return []

    out: list[Flowable] = []
    if caption:
        out.append(note(caption))
        out.append(Spacer(1, 4))

    cell_width = width / columns - 10
    cells: list[list[Flowable]] = []

    for person in signatories:
        # Room to actually sign in, above the rule.
        parts: list[Flowable] = [Spacer(1, 30), Rule(cell_width)]
        parts.append(Paragraph(person.name or "&nbsp;", CELL_BOLD))
        parts.append(Paragraph(person.role, CELL))
        if person.subtitle:
            parts.append(Paragraph(person.subtitle, NOTE))
        if person.date_line:
            parts.append(Spacer(1, 9))
            parts.append(Rule(cell_width * 0.6, thickness=0.5))
            parts.append(Paragraph("Date signed", NOTE))
        cells.append(parts)

    rows: list[list[object]] = []
    for i in range(0, len(cells), columns):
        row: list[object] = list(cells[i : i + columns])
        while len(row) < columns:
            row.append(Spacer(1, 1))
        rows.append(row)

    grid = Table(rows, colWidths=[width / columns] * columns)
    grid.setStyle(
        TableStyle(
            [
                ("VALIGN", (0, 0), (-1, -1), "TOP"),
                ("LEFTPADDING", (0, 0), (-1, -1), 0),
                ("RIGHTPADDING", (0, 0), (-1, -1), 10),
                ("TOPPADDING", (0, 0), (-1, -1), 6),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 10),
            ]
        )
    )
    out.append(grid)
    return out


def remarks_block(width: float, *, lines: int = 4, label: str = "Remarks") -> list[Flowable]:
    """Ruled space for whatever the form did not anticipate."""
    out: list[Flowable] = [heading(label)]
    for _ in range(lines):
        out.append(Spacer(1, 13))
        out.append(Rule(width, thickness=0.5, color=HAIRLINE))
    out.append(Spacer(1, 6))
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
