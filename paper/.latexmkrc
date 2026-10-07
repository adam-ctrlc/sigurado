# LuaLaTeX so that fontspec can load Arial from the system font directory,
# and biber because the bibliography is biblatex rather than bibtex.
$pdf_mode = 4;
$bibtex_use = 2;
$biber = 'biber %O %S';
