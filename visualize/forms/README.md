# Sign-off forms

Print-ready PDFs for the paperwork a capstone actually has to hand in and get
signed. Each script writes one form into `out/`.

```bash
cd visualize/forms
python generate_all.py          # everything
python bill_of_materials.py     # or just one
```

Needs `reportlab`. The generated PDFs are ignored by git; the scripts are the
source of truth.

## House style

Every form obeys the same four rules, set once in `layout.py`:

- **Arial, 12 point, everywhere.** Headings, table cells, captions and the title
  are all one size. Emphasis comes from weight and capitals, never from size.
  Arial is loaded from the system font file; on a machine without it the scripts
  fall back to Helvetica, which shares its metrics, and say so.
- **Portrait letter**, 0.6 inch margins on all four sides.
- **No footer.** Nothing on the page but the form.
- **Room to write.** Blank rows are 30 points tall, signature rules have 28
  points of clear space above them, and every labelled field gets its own row.

Twelve point is large for a form, which is why the tables here are four or five
columns at most. Anything that would need a sixth column is stacked inside a
cell instead: the unit sits with the quantity, the expected result sits under
the test steps, and yes/no boxes share one Result column.

## The forms

| Script | Form | What it is for |
| --- | --- | --- |
| `bill_of_materials.py` | Bill of Materials | 31 components in 7 sections, with quantities filled in and prices blank. Prepared, checked, recommended, approved. |
| `component_receipt.py` | Component Receipt and Inspection | One sheet per delivery: what arrived against what was ordered, and whether it survived the trip. |
| `test_witness.py` | Test Witness Sheet | 14 cases covering the two-scan sequence, the refusals, and the record that has to follow an opening. |
| `enrollment_consent.py` | Fingerprint Enrollment Consent | One per person: what is stored, who can see it, how to withdraw. |
| `acceptance_turnover.py` | Acceptance and Turnover | 10 acceptance criteria and an itemized handover, signed by whoever receives the system. |
| `approval_sheet.py` | Approval Sheet | The endorsement, the panel verdict, and the signatures a bound copy needs. |

## What is filled in and what is not

Filled in: anything that comes from the project itself. Component quantities and
specifications, the test cases, the acceptance criteria, the consent notice, the
group members, the subject and section.

Blank: anything that changes per copy, per quotation, per panel member or per
delivery. Prices, suppliers, dates, serial numbers, signatures, the school year,
and the adviser's name.

That split is deliberate. A form with the specifications already correct is
worth printing; a form that guesses at a price is worth nothing.

## Changing them

- **Project identity** lives in `project.py`. Set `school`, `department`,
  `adviser` and `school_year` there once and every form picks them up. Anything
  left as an empty string prints as a line to write on.
- **The look** lives in `layout.py`: page size, the font and its one size, the
  ruled table, the signature block. Change it there and all six forms follow.
- **A form's content** lives in its own script, near the top, as plain tuples.

## Adding a form

Copy the shape of the shortest one, `enrollment_consent.py`:

```python
import layout
from layout import FormDoc, build, output_dir

FORM_NAME = "..."
PURPOSE = "..."

def story(width: float) -> list[Flowable]:
    out = layout.title_block(FORM_NAME, PURPOSE)
    out.append(layout.identity_block(width))
    ...
    out.extend(layout.signature_block(PARTIES, width))
    return out

def main() -> None:
    path = output_dir() / "Sigurado_....pdf"
    doc = FormDoc(path, form_name=FORM_NAME)
    build(doc, story(doc.width))
```

Then add it to `FORMS` in `generate_all.py`.

Give a table its title with `data_table(..., title="Section name")` rather than
a separate heading above it. The title becomes a spanning row, so it can never
be stranded at the foot of a page, and it repeats with the column headers when
the table runs over.
