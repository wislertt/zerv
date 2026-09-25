# Preview GIF

Terminal preview shown in the README and docs index. Rendered with
[vhs](https://github.com/charmbracelet/vhs) 0.12.1 (0.12.0 is broken and
writes no output).

`zerv-preview.gif` shows `zerv flow` across four git states:

1. tagged release on `main` -> `1.2.3`
2. work lands on `develop` -> `1.2.4-beta.1.post.1`
3. feature branch off `develop` -> `1.2.4-alpha.<id>.post.2`
4. uncommitted changes -> same plus `.dev.<timestamp>`

## Still PNG

`zerv-preview.png` is a two-column summary of all four beats (beats 1-2
left, 3-4 right). It is composited from GIF frames, so re-derive it after
a re-render rather than regenerating it with the tape: pull the end-of-beat
frames (~7s, ~14s, ~33s in the current GIF) and stack beats 1+2 left of
beats 3+4 with a 3px `#3E525F` divider.

## Demo repo setup

The tape expects a demo repo with one worktree per git state, so the tape
never needs visible setup commands. From a scratch directory:

```bash
mkdir zerv-demo && cd zerv-demo && git init -q repo && cd repo
git config user.email demo@demo.dev
git config user.name Demo

printf 'zerv.tape\nzerv-preview.gif\n' > .gitignore
printf 'schema = "standard-no-context"\n' > zerv.toml
printf 'fn main() { println!("hello"); }\n' > main.rs
git add -A && git commit -qm "init" && git tag v1.2.3

git worktree add -q ../wt-develop -b develop
printf '// dev work\n' >> ../wt-develop/main.rs
git -C ../wt-develop commit -qam "dev work"

git worktree add -q ../wt-feature -b feature/otp develop
printf '// otp work\n' >> ../wt-feature/main.rs
git -C ../wt-feature commit -qam "add otp"

cp /path/to/zerv.tape .
vhs zerv.tape
```

`standard-no-context` strips the build context (`+branch.N.g<hash>`) so the
GIF only shows the version part.

## Re-rendering

Beat 4 appends to `wt-feature/main.rs` and leaves it dirty. Reset before
every render or beat 3 wrongly shows a `.dev` suffix:

```bash
git -C ../wt-feature checkout -q -- main.rs
vhs zerv.tape
```

## Tape notes (vhs 0.12.1)

- The tape's `Output` must stay `zerv-preview.gif` (or another gitignored
  name). vhs creates the output file when the render starts, so an
  untracked output makes zerv see the repo as dirty from the first frame
  and every beat shows a wrong version.
- `Hide` only pauses frame recording. Hidden commands still run in the
  visible terminal, so put `clear` plus a `Sleep 1s` inside the `Hide`
  block before `Show`: the sleep lets the recorder settle past the race
  at resume, otherwise the transition flashes.
- zsh has no interactive comments, so captions use `echo 'text'`.
- The tape parser does not support `\"` escapes; use single quotes inside
  double-quoted `Type` strings.
