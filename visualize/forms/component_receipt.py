"""Component receipt and inspection sheet.

Signed when parts arrive: what was ordered against what turned up, and whether
it survived the trip. Catching a dead sensor on delivery day is cheaper than
finding it during integration.

Run:  python component_receipt.py
"""

from reportlab.platypus import Flowable, Spacer

import layout
from layout import FormDoc, build, output_dir
from project import Signatory

FORM_NAME = "Component Receipt and Inspection"
PURPOSE = (
    "One sheet per delivery. Record what arrived, inspect it against the order, "
    "and note anything short, damaged or substituted before signing."
)

CHECKS: tuple[str, ...] = (
    "Quantities match the purchase order",
    "Packaging arrived intact and unopened",
    "Model and specification match the bill of materials",
    "No visible damage, bent pins or scorching",
    "Modules power up and are recognized over USB or I2C",
    "Fingerprint sensors respond and report a template count",
    "Warranty slips and receipts are attached",
)

PARTIES: tuple[Signatory, ...] = (
    Signatory("Received and inspected by", subtitle="Project member"),
    Signatory("Verified by", subtitle="Adviser or custodian"),
)


def story(width: float) -> list[Flowable]:
    out: list[Flowable] = []
    out.extend(layout.title_block(FORM_NAME, PURPOSE))
    out.append(
        layout.identity_block(
            width,
            extra=(("Supplier", ""), ("Delivery receipt no.", ""), ("Date received", "")),
        )
    )
    out.append(Spacer(1, 12))

    out.append(layout.heading("Items received"))
    out.append(
        layout.data_table(
            [
                "No.",
                "Item and specification",
                "Qty ordered",
                "Qty received",
                "Unit price",
                "Condition",
            ],
            [],
            [
                width * 0.05,
                width * 0.37,
                width * 0.12,
                width * 0.12,
                width * 0.14,
                width * 0.20,
            ],
            blank_rows=16,
            align_right=(4,),
        )
    )
    out.append(Spacer(1, 6))
    out.append(
        layout.note(
            "Copy the item numbers from the bill of materials so the two sheets line up."
        )
    )
    out.append(Spacer(1, 10))

    out.append(layout.heading("Inspection"))
    out.append(
        layout.data_table(
            ["Check", "Yes", "No", "Not applicable", "Remarks"],
            [[check, "[  ]", "[  ]", "[  ]", ""] for check in CHECKS],
            [width * 0.38, width * 0.07, width * 0.07, width * 0.13, width * 0.35],
        )
    )
    out.append(Spacer(1, 10))

    out.extend(
        layout.remarks_block(width, lines=4, label="Shortages, damage or substitutions")
    )
    out.append(Spacer(1, 6))
    out.append(
        layout.note(
            "Report anything short or damaged to the supplier within their return window. "
            "Note the date reported and the reference given: ____________________."
        )
    )
    out.append(Spacer(1, 12))

    out.extend(layout.signature_block(PARTIES, width))
    return out


def main() -> None:
    path = output_dir() / "Sigurado_Component_Receipt.pdf"
    doc = FormDoc(path, form_name=FORM_NAME)
    build(doc, story(doc.width))
    print(f"Saved {path.name}")


if __name__ == "__main__":
    main()
