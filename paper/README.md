# Sigurado project paper

LaTeX source for the CPE 048 final project paper. Builds with MiKTeX.

## Building

```bash
latexmk main.tex
```

That is the whole thing. `.latexmkrc` already selects LuaLaTeX and biber, and
latexmk runs as many passes as the document needs.

To clean up build artifacts:

```bash
latexmk -C
```

## If citations come out as `alrawili2024` with an empty parenthesis

The document is fine. You ran a single pass.

Citations are resolved by biber, which runs *between* LaTeX passes. On the first
pass there is no `.bbl` file yet, so every `\textcite` prints its raw key and an
empty marker. The sequence has to be:

```
lualatex main.tex
biber main
lualatex main.tex
lualatex main.tex
```

`latexmk` does all four for you, which is why it is the recommended command. If
you are building from a LaTeX editor, point its build profile at `latexmk`
rather than at a bare "LuaLaTeX" button, or press the button four times with a
biber run in the middle.

A finished build has no undefined citations. Check with:

```bash
grep -ci undefined main.log     # expect 0
```

## Format

Set in `main.tex`, and deliberately not APA defaults where the two disagree:

| Setting | Value | Notes |
| --- | --- | --- |
| Paper | Letter, 8.5 x 11 in | Short bond |
| Font | Arial 12 | Real Arial, embedded; this is why the build uses LuaLaTeX |
| Margins | 2 in left, 1 in top/right/bottom | Left margin is for binding; APA default is 1 in all round |
| Alignment | Justified | Departs from APA 7, which specifies flush left / ragged right |
| Hyphenation | Off | Whole words wrap; only real hyphens (`append-only`) may break |
| Spacing | Double | apa7 class default |
| Headings | `CHAPTER N: TITLE`, then `N.1`, `N.1.1` | APA does not number headings; the numbering is added on top |
| Floats | In text | `floatsintext`, otherwise apa7 defers them all to the end |

Each chapter starts on a new page.

## Layout

```
main.tex              preamble, title block, format settings
references.bib        bibliography, 7 entries
sections/             one file per chapter
figures/             (empty; for the 3D render, see below)
.latexmkrc            engine and bibliography backend
```

## Outstanding

**Figure 3 is a placeholder.** The guide requires the hardware architecture in
3D. A parametric model of the cabinet and scanner unit already exists in
`../visualize/scad/`, but OpenSCAD is not installed on this machine, so it could
not be rendered. Once OpenSCAD is available:

```bash
openscad -o paper/figures/hardware-3d.png --imgsize=1600,1200 visualize/scad/main.scad
```

Then replace the `\framebox` placeholder in
`sections/02-system-design.tex` with:

```latex
\includegraphics[width=0.85\linewidth]{figures/hardware-3d.png}
```

## Sources

Seven, all 2023 or later, each one checked against the publisher or arXiv record
before it was added. Nothing in `references.bib` was written from memory.
