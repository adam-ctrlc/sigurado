"""Bill of materials with a sign-off block.

Quantities and specifications come from what the system actually needs, so the
sheet is useful before anybody has priced anything. Prices, suppliers and the
signatures are left blank: those change per quotation and per copy.

Run:  python bill_of_materials.py
"""

from reportlab.lib.units import inch
from reportlab.platypus import Flowable, Paragraph, Spacer

import layout
from layout import CELL, CELL_BOLD, FormDoc, build, output_dir
from project import LineItem, Section, Signatory

FORM_NAME = "Bill of Materials"
PURPOSE = (
    "Itemized list of components for one complete installation: one door node, "
    "one cabinet node, and the lock hardware. Fill in the unit price and supplier "
    "per quotation, then have the sheet checked and approved before ordering."
)

SECTIONS: tuple[Section, ...] = (
    Section(
        "A. Controllers and biometrics",
        (
            LineItem("ESP32 DevKit v1", "30-pin, ESP32-WROOM-32, USB micro-B", "2", "pc"),
            LineItem(
                "Fingerprint sensor",
                "R307 or AS608, optical, UART 57600 baud",
                "2",
                "pc",
            ),
            LineItem("16x2 LCD with I2C backpack", "PCF8574, address 0x27", "2", "pc"),
        ),
    ),
    Section(
        "B. Lock and door sensing",
        (
            LineItem("Solenoid lock", "12V DC, fail-secure, 0.6A holding", "1", "pc"),
            LineItem("Relay module", "1-channel, 5V logic, opto-isolated", "1", "pc"),
            LineItem(
                "Lever limit switch",
                "SPDT microswitch with roller lever, COM and NO used",
                "1",
                "pc",
            ),
            LineItem("Flyback diode", "1N4007, across the solenoid coil", "2", "pc"),
            LineItem("Strike plate and mounting bracket", "For the cabinet door", "1", "set"),
        ),
    ),
    Section(
        "C. Alerting",
        (
            LineItem("SIM800L GSM module", "Quad-band, with antenna", "1", "pc"),
            LineItem(
                "SIM800L power supply",
                "4.0V regulated, 2A peak capable, separate from the ESP32",
                "1",
                "pc",
            ),
            LineItem("Electrolytic capacitor", "1000uF 16V, across the SIM800L supply", "1", "pc"),
            LineItem("Prepaid SIM card", "With load for text messages", "1", "pc"),
        ),
    ),
    Section(
        "D. Panel and indicators",
        (
            LineItem("Active buzzer", "5V, for the door-left-open reminder", "1", "pc"),
            LineItem("LED indicator", "5mm, with 220 ohm resistor", "2", "pc"),
            LineItem("Momentary push button", "Panel mount, for enrollment", "2", "pc"),
            LineItem("Resistors", "220 ohm and 10k ohm assortment", "1", "set"),
        ),
    ),
    Section(
        "E. Power and wiring",
        (
            LineItem("Power supply", "12V 2A, for the solenoid", "1", "pc"),
            LineItem("Power supply", "5V 2A, for the controllers", "2", "pc"),
            LineItem("DC barrel jack with terminals", "5.5 x 2.1 mm", "3", "pc"),
            LineItem("Jumper wires", "Male-to-female and male-to-male", "2", "set"),
            LineItem("Stranded hook-up wire", "22 AWG, assorted colors", "1", "roll"),
            LineItem("Heat shrink tubing", "Assorted diameters", "1", "set"),
            LineItem("Screw terminal blocks", "2-pin and 3-pin, 5mm pitch", "1", "set"),
        ),
    ),
    Section(
        "F. Enclosure and mounting",
        (
            LineItem("Project enclosure", "For the door node, with cutouts", "1", "pc"),
            LineItem("Project enclosure", "For the cabinet node, with cutouts", "1", "pc"),
            LineItem("Cable gland", "12mm, for the enclosure entry", "2", "pc"),
            LineItem("Standoffs and screws", "M3, nylon and metal assortment", "1", "set"),
            LineItem("Perfboard or PCB", "For the interface wiring", "2", "pc"),
        ),
    ),
    Section(
        "G. Consumables and tools",
        (
            LineItem("Solder", "60/40 rosin core, 0.8mm", "1", "roll"),
            LineItem("Isopropyl alcohol", "For cleaning boards and the sensor face", "1", "btl"),
            LineItem("Cable ties", "Assorted lengths", "1", "pack"),
        ),
    ),
)

APPROVERS: tuple[Signatory, ...] = (
    Signatory("Prepared by", subtitle="Project leader"),
    Signatory("Checked by", subtitle="Technical adviser"),
    Signatory("Recommending approval", subtitle="Program chair"),
    Signatory("Approved by", subtitle="Dean or authorized signatory"),
)


def _section_rows(section: Section, start: int) -> tuple[list[list[str]], int]:
    """Rows for one section, numbered continuously across the whole sheet."""
    rows: list[list[str]] = []
    number = start
    for item in section.items:
        rows.append(
            [
                str(number),
                f"<b>{item.item}</b><br/>{item.spec}",
                item.quantity,
                item.unit,
                item.unit_price,
                "",
                item.supplier,
            ]
        )
        number += 1
    return rows, number


def story(width: float) -> list[Flowable]:
    out: list[Flowable] = []
    out.extend(layout.title_block(FORM_NAME, PURPOSE))
    out.append(layout.identity_block(width, extra=(("Quotation ref.", ""),)))
    out.append(Spacer(1, 10))

    header = ["No.", "Item and specification", "Qty", "Unit", "Unit price", "Amount", "Supplier"]
    widths = [
        width * 0.05,
        width * 0.40,
        width * 0.06,
        width * 0.07,
        width * 0.13,
        width * 0.13,
        width * 0.16,
    ]

    number = 1
    for section in SECTIONS:
        rows, number = _section_rows(section, number)
        out.append(layout.heading(section.heading))
        out.append(
            layout.data_table(header, rows, widths, blank_rows=1, align_right=(4, 5))
        )
        out.append(Spacer(1, 4))

    out.append(layout.heading("Totals"))
    out.append(
        layout.data_table(
            ["Description", "Amount"],
            [
                ["Subtotal", ""],
                ["Shipping and handling", ""],
                ["Contingency (10 percent)", ""],
                ["<b>Grand total</b>", ""],
            ],
            [width * 0.72, width * 0.28],
            align_right=(1,),
        )
    )
    out.append(Spacer(1, 8))
    out.append(
        layout.note(
            "Currency: ____________.  Prices are valid until ____________.  "
            "Attach the supplier quotation to this sheet."
        )
    )
    out.append(Spacer(1, 12))

    out.extend(
        layout.signature_block(
            APPROVERS,
            width,
            caption=(
                "By signing, each party confirms the quantities and specifications above "
                "are correct and that the total is within the approved budget."
            ),
        )
    )
    return out


def main() -> None:
    path = output_dir() / "Sigurado_Bill_of_Materials.pdf"
    doc = FormDoc(path, form_name=FORM_NAME)
    build(doc, story(doc.width))
    total = sum(len(section.items) for section in SECTIONS)
    print(f"Saved {path.name} ({total} items across {len(SECTIONS)} sections)")


if __name__ == "__main__":
    main()
