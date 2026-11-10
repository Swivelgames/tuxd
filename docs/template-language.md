# Template grammar (draft)

[template-language.ebnf](template-language.ebnf) uses **W3C EBNF**, with `::=`,
postfix `*`, `+`, `?`, and `#xNN` code points. Select W3C EBNF when importing
it into grammar tools; it is not BNF, PEG, Instaparse, or ISO 14977 syntax.
The grammar file contains no tool-specific prose character classes or comments.

The input is raw UTF-8 template source. Decode escaped source before parsing.
No whitespace is implicitly skipped. ANSI interpretation and allowed output
controls are rendering policies, not structural syntax rules.

## Bytes

| Character | Hex | Purpose |
| --- | --- | --- |
| SOH | 01 | Helper header begins |
| STX | 02 | Block body begins |
| DLE | 10 | Escape a reserved byte |
| EM | 19 | Inline helper or block ends |
| FS | 1C | Include path delimiter |
| US | 1F | Value-reference delimiter |
| ACK, GS, RS | 06, 1D, 1E | Reserved; must be escaped to occur literally |

## Evaluation rules outside the grammar

- Paths are case-sensitive dotted identifiers. Text substitution requires strings.
- Header whitespace separates arguments and produces no output. References delimit
	themselves, so `each<US>users<US>user` has two arguments. Literal words are
	maximal tokens. Quoted arguments and helpers inside headers are unsupported.
- Required block helpers are `if(reference)`, `unless(reference)`, and
	`each(reference, alias)`. The alias is a literal identifier.
- Required inline helper `count(reference)` accepts an array.
- Helper existence, form, argument counts and types are validated separately.
	Extensions must be registered explicitly.
- `if` renders for true or populated strings/arrays; `unless` inverts this test.
	Missing paths are false; objects are type errors. Missing text values are errors.
- Loop aliases are local and may shadow outer bindings. Otherwise lookup uses the
	root model; no implicit parent-context search occurs.
- Includes inherit bindings but are independently balanced templates. Resolution,
	missing includes and cycle errors belong to the renderer.
- Syntax is checked in skipped bodies, but their values/includes are not evaluated.
- Body whitespace is preserved exactly. Example text is preview-only metadata.
- Escaped bytes, substituted values and helper results are never reparsed as syntax.
- The renderer owns style preservation/restoration and nesting/output limits.

## Validation

Validated with the W3C grammar reader and generated parser in npm `ebnf@1.9.1`:
seven valid examples and eleven invalid examples. Cases include nested helpers,
Unicode, includes, escaping, adjacent close/open helpers, incomplete delimiters,
reserved bytes, invalid paths and forbidden helpers inside headers.

To check grammar syntax and compile it without adding project dependencies:

```sh
npm install --prefix /tmp/tuxd-ebnf-validation ebnf@1.9.1 --no-audit --no-fund
node - docs/template-language.ebnf <<'JS'
const fs = require('node:fs');
const assert = require('node:assert/strict');
const { Grammars } = require('/tmp/tuxd-ebnf-validation/node_modules/ebnf');
const source = fs.readFileSync(process.argv[2], 'utf8');
const ast = Grammars.W3C.defaultParser.getAST(source);
assert(ast && !ast.errors.length && ast.text === source);
new Grammars.W3C.Parser(source);
console.log('W3C EBNF syntax and parser compilation passed');
JS
```

This checks structural grammar, not the unimplemented rendering semantics.
