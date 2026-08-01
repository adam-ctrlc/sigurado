"""Build every form into `out/`.

Run:  python generate_all.py
"""

from collections.abc import Callable

import acceptance_turnover
import approval_sheet
import bill_of_materials
import component_receipt
import enrollment_consent
import test_witness
from layout import output_dir

FORMS: tuple[tuple[str, Callable[[], None]], ...] = (
    ("Bill of materials", bill_of_materials.main),
    ("Component receipt and inspection", component_receipt.main),
    ("Test witness sheet", test_witness.main),
    ("Fingerprint enrollment consent", enrollment_consent.main),
    ("Acceptance and turnover", acceptance_turnover.main),
    ("Approval sheet", approval_sheet.main),
)


def main() -> None:
    for label, build_form in FORMS:
        print(f"[{label}]")
        build_form()
    print(f"\nAll {len(FORMS)} forms are in {output_dir()}")


if __name__ == "__main__":
    main()
