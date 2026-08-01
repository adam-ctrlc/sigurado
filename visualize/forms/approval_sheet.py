"""Approval sheet for the panel and the adviser.

The page a capstone is bound with: the endorsement, the verdict, and the
signatures of everyone who has to agree the project passed.

Run:  python approval_sheet.py
"""

from reportlab.platypus import Flowable, Paragraph, Spacer

import layout
from layout import BODY, FormDoc, build, output_dir
from project import PROJECT, Signatory

FORM_NAME = "Approval Sheet"
PURPOSE = (
    "Presented to the panel in partial fulfillment of the requirements for "
    f"{PROJECT.subject}. Complete the verdict, then have each signatory sign and date."
)

ENDORSEMENT = (
    "This capstone project, <b>{title}</b>, prepared and submitted by "
    "<b>{members}</b>, in partial fulfillment of the requirements for "
    "{subject}, has been examined and is recommended for oral examination."
)

ACCEPTANCE = (
    "Approved by the panel of examiners on oral examination held on "
    "____________________ with a grade of ____________."
)

VERDICTS = (
    "Approved as presented",
    "Approved subject to the revisions noted below",
    "Not approved; resubmission required",
)

PANEL: tuple[Signatory, ...] = (
    Signatory("Panel member", subtitle="Signature over printed name"),
    Signatory("Panel member", subtitle="Signature over printed name"),
    Signatory("Panel member", subtitle="Signature over printed name"),
    Signatory("Panel chair", subtitle="Signature over printed name"),
)

ENDORSERS: tuple[Signatory, ...] = (
    Signatory("Adviser", name=PROJECT.adviser, subtitle="Signature over printed name"),
    Signatory("Program chair", subtitle="Signature over printed name"),
)


def story(width: float) -> list[Flowable]:
    out: list[Flowable] = []
    out.extend(layout.title_block(FORM_NAME, PURPOSE))
    out.extend(layout.identity_block(width, extra=(("Date of defense", ""),)))
    out.append(Spacer(1, 18))

    out.append(layout.heading("Endorsement"))
    out.append(
        Paragraph(
            ENDORSEMENT.format(
                title=PROJECT.title,
                members="; ".join(PROJECT.member_lines()),
                subject=PROJECT.subject,
            ),
            BODY,
        )
    )
    out.append(Spacer(1, 10))
    out.extend(layout.signature_block(ENDORSERS, width))

    out.append(Spacer(1, 6))
    out.append(
        layout.data_table(
            ["Mark one", "Verdict"],
            [["[      ]", verdict] for verdict in VERDICTS],
            [width * 0.18, width * 0.82],
            title="Verdict of the panel",
        )
    )
    out.append(Spacer(1, 8))
    out.append(Paragraph(ACCEPTANCE, BODY))
    out.append(Spacer(1, 12))

    out.extend(
        layout.signature_block(
            PANEL,
            width,
            caption="Panel of examiners",
        )
    )

    out.extend(layout.remarks_block(width, lines=5, label="Revisions required"))
    out.append(Spacer(1, 8))
    out.append(
        layout.note(
            "Attach the revised copy and this sheet when resubmitting. "
            "Retain one signed original for the department file."
        )
    )
    return out


def main() -> None:
    path = output_dir() / "Sigurado_Approval_Sheet.pdf"
    doc = FormDoc(path, form_name=FORM_NAME)
    build(doc, story(doc.width))
    print(f"Saved {path.name}")


if __name__ == "__main__":
    main()
