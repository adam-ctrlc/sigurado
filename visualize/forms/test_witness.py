"""Test witness sheet.

The cases are the ones that decide whether the system does what it claims:
the two-scan sequence, the refusals it exists to produce, and the record that
has to follow an opening. A witness marks each pass or fail and signs.

Run:  python test_witness.py
"""

from dataclasses import dataclass

from reportlab.platypus import Flowable, Spacer

import layout
from layout import FormDoc, build, output_dir
from project import Signatory

FORM_NAME = "Test Witness Sheet"
PURPOSE = (
    "Each case is run in front of a witness on the assembled hardware. Mark pass "
    "or fail, note what was actually observed, and sign at the bottom. A failed "
    "case needs a remark and a retest date."
)


@dataclass(frozen=True)
class TestCase:
    reference: str
    title: str
    steps: str
    expected: str


CASES: tuple[TestCase, ...] = (
    TestCase(
        "T-01",
        "Door opens for a registered finger",
        "Present an enrolled finger to the door reader.",
        "Lock releases, LCD names the person, and a door window opens.",
    ),
    TestCase(
        "T-02",
        "Door refuses an unknown finger",
        "Present a finger that was never enrolled.",
        "Lock stays shut, LCD reads Not registered, and the refusal is logged.",
    ),
    TestCase(
        "T-03",
        "Cabinet opens for the same person",
        "Scan at the door, then at the cabinet within the window.",
        "Cabinet unlocks and shows a six character code.",
    ),
    TestCase(
        "T-04",
        "Cabinet refuses without a door scan",
        "Present an enrolled finger at the cabinet with no door scan first.",
        "Cabinet stays shut and the event is logged as a tailgating attempt.",
    ),
    TestCase(
        "T-05",
        "Cabinet refuses after the window closes",
        "Scan the door, wait past the session window, then scan the cabinet.",
        "Cabinet stays shut and the reason reads that the window expired.",
    ),
    TestCase(
        "T-06",
        "Enrollment binds only after claiming",
        "Enroll a finger at a reader, then claim its code on the website.",
        "Before claiming the finger opens nothing; after claiming it is named on events.",
    ),
    TestCase(
        "T-07",
        "Checkout requires the cabinet code",
        "Record a checkout using the code shown at the opening.",
        "The record is accepted and appears in the list with the items typed in.",
    ),
    TestCase(
        "T-08",
        "Checkout without a valid code is refused",
        "Try to record a checkout with a made-up code.",
        "The record is refused and the attempt is logged as a refused code.",
    ),
    TestCase(
        "T-09",
        "Door left open is noticed",
        "Open the cabinet and leave the door ajar past the alarm threshold.",
        "The panel asks for the door to be closed and the buzzer sounds.",
    ),
    TestCase(
        "T-10",
        "Text alert on a cabinet opening",
        "Open the cabinet with at least one alert recipient configured.",
        "A text arrives on the recipient's number within a reasonable time.",
    ),
    TestCase(
        "T-11",
        "A quiet reader is reported",
        "Unplug a reader and wait past the offline threshold.",
        "The devices page shows it offline, and a flag is raised.",
    ),
    TestCase(
        "T-12",
        "A disabled account is refused",
        "Disable an account, then present that person's finger at the door.",
        "The door stays shut and the refusal names the disabled account.",
    ),
    TestCase(
        "T-13",
        "An unrecorded opening is flagged",
        "Open the cabinet and deliberately do not record a checkout.",
        "Once the code expires the flags page reports an unrecorded opening.",
    ),
    TestCase(
        "T-14",
        "The trail cannot be edited",
        "Attempt to change or delete a reader event as an administrator.",
        "No interface or endpoint allows it; the event stands as recorded.",
    ),
)

WITNESSES: tuple[Signatory, ...] = (
    Signatory("Tested by", subtitle="Project member"),
    Signatory("Witnessed by", subtitle="Adviser or lab custodian"),
)


def story(width: float) -> list[Flowable]:
    out: list[Flowable] = []
    out.extend(layout.title_block(FORM_NAME, PURPOSE))
    out.append(
        layout.identity_block(
            width, extra=(("Firmware version", ""), ("Test location", ""))
        )
    )
    out.append(Spacer(1, 10))

    header = ["Ref.", "Case and steps", "Expected result", "Pass", "Fail", "Observed"]
    widths = [
        width * 0.05,
        width * 0.28,
        width * 0.27,
        width * 0.05,
        width * 0.05,
        width * 0.30,
    ]
    rows = [
        [
            case.reference,
            f"<b>{case.title}</b><br/>{case.steps}",
            case.expected,
            "[  ]",
            "[  ]",
            "",
        ]
        for case in CASES
    ]

    out.append(layout.heading("Test cases"))
    out.append(layout.data_table(header, rows, widths, blank_rows=2))
    out.append(Spacer(1, 8))

    out.append(layout.heading("Summary"))
    out.append(
        layout.data_table(
            ["Cases run", "Passed", "Failed", "Retest scheduled"],
            [["", "", "", ""]],
            [width * 0.25] * 4,
        )
    )
    out.append(Spacer(1, 10))

    out.extend(layout.remarks_block(width, lines=4, label="Notes on failures"))
    out.append(Spacer(1, 8))
    out.extend(
        layout.signature_block(
            WITNESSES,
            width,
            caption=(
                "The witness confirms these cases were run on the assembled hardware "
                "in their presence, and that the results above are what happened."
            ),
        )
    )
    return out


def main() -> None:
    path = output_dir() / "Sigurado_Test_Witness_Sheet.pdf"
    doc = FormDoc(path, form_name=FORM_NAME, orientation="landscape")
    build(doc, story(doc.width))
    print(f"Saved {path.name} ({len(CASES)} cases)")


if __name__ == "__main__":
    main()
