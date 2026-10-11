# Contributing a translation

LuLu's interface text lives in this folder as plain JSON files, one per
language. You do not need to install anything or run the app to contribute a
translation - any text editor works.

## Files

| File       | Purpose                                                  |
| ---------- | -------------------------------------------------------- |
| `en.json`  | English. This is the reference: every key must exist here. |
| `id.json`  | Bahasa Indonesia.                                        |
| `<xx>.json`| Your language: copy `en.json` and translate the values.  |

## Adding a new language

1. Copy `en.json` to `<language-code>.json`. The code is the ISO 639-1 code:
   `fr` for French, `de` for German, `ms` for Malay, and so on.
2. Translate every **value** (the text on the right of the colon). Leave the
   **keys** (the text on the left) exactly as they are.
3. Keep anything inside `{curly braces}` untouched - that is a placeholder the
   app fills in, for example `{current}` in `"rowsOf": "of {current} rows"`.
4. Send the file as a pull request. A maintainer wires the new locale into
   `nuxt.config.ts` (three lines).

## Fixing an existing translation

Edit the value in place and send a pull request. One language per pull request
keeps review simple.

## Rules

- Translate the meaning, not word by word. Buttons are short on purpose.
- Keys are grouped by page (`nav`, `login`, `report`, ...). Keys inside the
  `common` group are reused everywhere - changing one changes many screens.
- If a key is missing in your file, the app shows the English text instead of
  an error. Partial translations are welcome: translate what you can, the rest
  falls back to English.
- `en.json` is the source of truth. When the app gains a new screen, its keys
  are added to `en.json` first and other languages catch up afterwards.

## Checking your work

```bash
cd LuLu-FrontEnd
python3 -c "
import json
# every file must parse and must hold exactly the keys en.json holds
ref = set(json.load(open('i18n/locales/en.json')).keys())
for lang in ['id']:
    got = set(json.load(open(f'i18n/locales/{lang}.json')).keys())
    assert got == ref, f'{lang}: missing {ref - got}, extra {got - ref}'
print('groups match')
"
```
