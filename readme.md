# Rust, Bevy based guitarist's helper

This projects is aimed to help guitarists to convert any tabs to any tuning they want. It also can help beginner guitarists to learn notes, chords and experiment with different tunings.

## Features

- it can show all 6 strings with notes on different frets

- by default standard tuning (`E A D G B E`)

- you can change tunings to different tunings, for example `DROP D` (When you turn 6th string down one step from `E` to `D`)

- you can change any string to any note you want, so you can make any tuning you want

- you can play each note on any string and fret

## GitHub Pages

Site: https://bronzecrab.github.io/guitar-notes/

After the Actions workflow deploys, set once in the repo:

**Settings → Pages → Build and deployment → Source → GitHub Actions**

If Source stays on “Deploy from a branch” (`main`), the browser will keep loading the repo root `index.html` and `guitar_notes.js` will 404 even when the workflow is green.

How to run locally wasm:
```ps1
.\serve-wasm.ps1
```