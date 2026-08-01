"""Who and what every form is about.

Edit this once and every generated form follows. Anything left as an empty
string prints as a blank line to fill in by hand, which is deliberate: the
fields that change per copy, per panel member and per delivery are better
written in ink than baked into a PDF.
"""

from dataclasses import dataclass, field


@dataclass(frozen=True)
class Project:
    title: str
    short_name: str
    subject: str
    section: str
    school: str
    department: str
    members: tuple[str, ...]
    adviser: str
    school_year: str

    def member_lines(self) -> list[str]:
        """Members as they should appear on a form, one per line."""
        return list(self.members)


PROJECT = Project(
    title=(
        "Sigurado: Dual-Biometric Sequential Access and "
        "Materials Accountability System"
    ),
    short_name="Sigurado",
    subject="CPE 048: Embedded Systems",
    section="COC-FA-CPE4-01",
    # Left blank on purpose: fill in on the printed copy, or set them here if
    # every form should carry the same wording.
    school="",
    department="",
    members=(
        "Aballa, Gerjhon",
        "Abragan, John Lloyd",
        "Cuenca, John Adam A.",
    ),
    adviser="",
    school_year="",
)


@dataclass(frozen=True)
class Signatory:
    """One line to sign on.

    `name` and `date_line` are usually left empty so the form prints a rule to
    write on. Fill `name` in only when the same person signs every copy.
    """

    role: str
    name: str = ""
    subtitle: str = ""
    date_line: bool = True


@dataclass(frozen=True)
class LineItem:
    """A row of the bill of materials.

    Quantities are known from the build; prices are not, so they print blank.
    """

    item: str
    spec: str
    quantity: str = ""
    unit: str = "pc"
    unit_price: str = ""
    supplier: str = ""


@dataclass(frozen=True)
class Section:
    """A titled group of line items, so the bill reads by subsystem."""

    heading: str
    items: tuple[LineItem, ...] = field(default_factory=tuple)
