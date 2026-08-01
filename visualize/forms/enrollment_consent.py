"""Fingerprint enrollment consent.

The system reads a fingerprint, so the person it belongs to should agree to it
in writing and be told plainly what is stored. One sheet per person, kept by
the laboratory.

Run:  python enrollment_consent.py
"""

from reportlab.platypus import Flowable, Paragraph, Spacer

import layout
from layout import BODY, FormDoc, build, output_dir
from project import Signatory

FORM_NAME = "Fingerprint Enrollment Consent"
PURPOSE = (
    "One sheet per person enrolled. Read the notice, complete the details, and "
    "sign before enrollment. The laboratory keeps the signed original."
)

NOTICE: tuple[tuple[str, str], ...] = (
    (
        "What is stored",
        "The reader converts your fingerprint into a template and keeps that "
        "template in its own memory. The template is not a picture of your "
        "fingerprint and cannot be turned back into one.",
    ),
    (
        "What the server sees",
        "The server never receives your fingerprint or its template. It stores an "
        "opaque token that only identifies which slot on which reader matched.",
    ),
    (
        "What is recorded about you",
        "Your name, the reader, and the time, every time you scan; whether access "
        "was granted or refused; and what you record having taken from the cabinet.",
    ),
    (
        "Who can see it",
        "Laboratory staff and administrators. You can see your own records at any "
        "time by signing in to the website.",
    ),
    (
        "How long it is kept",
        "For as long as you have access, plus the retention period the laboratory "
        "sets: ____________________.",
    ),
    (
        "Withdrawing",
        "You may ask for your enrollment to be removed at any time. Your fingerprint "
        "template is erased from the reader; the past record of scans remains, "
        "because the trail cannot be edited.",
    ),
)

DECLARATION = (
    "I have read the notice above. I understand what is stored, who can see it, and "
    "how to withdraw. I consent to enrolling my fingerprint for access to the "
    "laboratory stockroom."
)

MINOR_NOTE = (
    "If the person enrolling is under the age of majority, a parent or guardian "
    "signs as well."
)

PARTIES: tuple[Signatory, ...] = (
    Signatory("Person enrolling", subtitle="Signature over printed name"),
    Signatory("Parent or guardian", subtitle="If applicable"),
    Signatory("Enrolled by", subtitle="Administrator who bound the finger"),
    Signatory("Noted by", subtitle="Laboratory custodian"),
)


def story(width: float) -> list[Flowable]:
    out: list[Flowable] = []
    out.extend(layout.title_block(FORM_NAME, PURPOSE))
    out.append(Spacer(1, 4))

    out.append(layout.heading("Person enrolling"))
    out.append(
        layout.data_table(
            ["Full name", "Student or employee no.", "Role", "Contact number"],
            [["", "", "", ""]],
            [width * 0.34, width * 0.22, width * 0.20, width * 0.24],
        )
    )
    out.append(Spacer(1, 4))
    out.append(
        layout.data_table(
            ["Username issued", "Fingers enrolled", "Readers", "Date enrolled"],
            [["", "", "Door [  ]   Cabinet [  ]", ""]],
            [width * 0.28, width * 0.22, width * 0.28, width * 0.22],
        )
    )
    out.append(Spacer(1, 12))

    out.append(layout.heading("What you are agreeing to"))
    out.append(
        layout.data_table(
            ["", ""],
            [[f"<b>{title}</b>", text] for title, text in NOTICE],
            [width * 0.26, width * 0.74],
            show_header=False,
        )
    )
    out.append(Spacer(1, 12))

    out.append(layout.heading("Declaration"))
    out.append(Paragraph(DECLARATION, BODY))
    out.append(Spacer(1, 6))
    out.append(layout.note(MINOR_NOTE))
    out.append(Spacer(1, 14))

    out.extend(layout.signature_block(PARTIES, width))

    out.append(Spacer(1, 4))
    out.append(layout.heading("Withdrawal, if any"))
    out.append(
        layout.data_table(
            ["Date withdrawn", "Template erased by", "Reason", "Signature"],
            [["", "", "", ""]],
            [width * 0.20, width * 0.26, width * 0.29, width * 0.25],
        )
    )
    return out


def main() -> None:
    path = output_dir() / "Sigurado_Enrollment_Consent.pdf"
    doc = FormDoc(path, form_name=FORM_NAME)
    build(doc, story(doc.width))
    print(f"Saved {path.name}")


if __name__ == "__main__":
    main()
