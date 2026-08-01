"""Acceptance and turnover sheet.

Signed when the system stops being a project and starts being something the
lab depends on. The criteria are the promises the system makes, so accepting
it means agreeing each one was demonstrated.

Run:  python acceptance_turnover.py
"""

from reportlab.platypus import Flowable, Paragraph, Spacer

import layout
from layout import BODY, FormDoc, build, output_dir
from project import PROJECT, Signatory

FORM_NAME = "Acceptance and Turnover"
PURPOSE = (
    "Completed when the system is handed to the laboratory. Each criterion is "
    "demonstrated in front of the receiving party before the sheet is signed."
)

STATEMENT = (
    "The undersigned confirm that <b>{title}</b> was demonstrated in full, that the "
    "criteria below were met, and that the system, its documentation and its "
    "credentials are turned over to the receiving party on the date signed."
)

CRITERIA: tuple[tuple[str, str], ...] = (
    (
        "Sequential access works",
        "A door scan opens a window, and the cabinet opens only for that same "
        "person while it is live.",
    ),
    (
        "Tailgating is refused",
        "A registered finger at the cabinet with no door scan behind it is turned "
        "away and recorded.",
    ),
    (
        "Enrollment is complete",
        "Every person who needs access is enrolled at both readers and named "
        "correctly on their events.",
    ),
    (
        "Checkouts require the cabinet code",
        "A record cannot be written without the code shown at a real opening.",
    ),
    (
        "The audit trail is readable",
        "Staff can search the log, read the sheet, and export a CSV without help.",
    ),
    (
        "Flags are understood",
        "The receiving party knows what each check means and what to do about it.",
    ),
    (
        "Alerts reach a real phone",
        "A test message was received by at least one listed recipient.",
    ),
    (
        "Readers report in",
        "Both nodes show as live, and the receiving party knows where to check.",
    ),
    (
        "Administrator handover",
        "An administrator account belonging to the laboratory exists, and its "
        "password has been changed from the seeded one.",
    ),
    (
        "Documentation handed over",
        "Wiring, device secrets procedure, backup instructions and the guides in "
        "the app were walked through.",
    ),
)

TURNED_OVER: tuple[tuple[str, str], ...] = (
    ("Door node, assembled and mounted", ""),
    ("Cabinet node, assembled and mounted", ""),
    ("Server or host machine running the backend", ""),
    ("Administrator credentials, handed over separately", ""),
    ("Device secrets procedure and rotation instructions", ""),
    ("Source code repository address", ""),
    ("Printed guides for administrators, faculty and students", ""),
    ("Spare components and consumables", ""),
)

PARTIES: tuple[Signatory, ...] = (
    Signatory("Turned over by", subtitle="Project leader"),
    Signatory("Received by", subtitle="Laboratory custodian or end user"),
    Signatory("Noted by", name=PROJECT.adviser, subtitle="Adviser"),
    Signatory("Approved by", subtitle="Program chair"),
)


def story(width: float) -> list[Flowable]:
    out: list[Flowable] = []
    out.extend(layout.title_block(FORM_NAME, PURPOSE))
    out.extend(
        layout.identity_block(
            width, extra=(("Turnover date", ""), ("Location or room", ""))
        )
    )
    out.append(Spacer(1, 16))

    out.append(Paragraph(STATEMENT.format(title=PROJECT.title), BODY))
    out.append(Spacer(1, 10))

    out.append(
        layout.data_table(
            ["No.", "Criterion", "Met", "Remarks"],
            [
                [str(i), f"<b>{name}</b><br/>{detail}", "Yes [  ]<br/>No [  ]", ""]
                for i, (name, detail) in enumerate(CRITERIA, start=1)
            ],
            [width * 0.07, width * 0.48, width * 0.15, width * 0.30],
            title="Acceptance criteria",
        )
    )
    out.append(Spacer(1, 10))

    out.append(
        layout.data_table(
            ["Item", "Qty", "Serial or tag", "Condition"],
            [[name, qty, "", ""] for name, qty in TURNED_OVER],
            [width * 0.46, width * 0.10, width * 0.24, width * 0.20],
            title="Items turned over",
            blank_rows=3,
        )
    )
    out.append(Spacer(1, 10))

    out.extend(layout.remarks_block(width, lines=4, label="Conditions or pending items"))
    out.append(Spacer(1, 6))
    out.append(
        layout.note(
            "Where a criterion is not met, acceptance is conditional until the pending "
            "item above is closed and initialed by both parties."
        )
    )
    out.append(Spacer(1, 12))

    out.extend(layout.signature_block(PARTIES, width))
    return out


def main() -> None:
    path = output_dir() / "Sigurado_Acceptance_and_Turnover.pdf"
    doc = FormDoc(path, form_name=FORM_NAME)
    build(doc, story(doc.width))
    print(f"Saved {path.name} ({len(CRITERIA)} criteria)")


if __name__ == "__main__":
    main()
