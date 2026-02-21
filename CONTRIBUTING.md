# Contributing to Qlue-ls

Thanks for your interest in improving Qlue-ls.

This project values high-quality, well-reasoned contributions over volume.  
Please read this document before opening an issue or pull request.

---

## Before You Start

- Check existing issues before opening a new one.
- For larger changes, open a discussion first.
- Keep pull requests focused and minimal.
- Refer to the [README](./README.md) for setup, installation, and build instructions.
- If you're unsure — ask.

---

## What Makes a Good Contribution

### ✔ Small and focused

One PR = one logical change.

### ✔ Clearly motivated

Explain:
- What problem does this solve?
- Why is this the right solution?
- What trade-offs were considered?

### ✔ Tested

Add tests where appropriate.

Bug fixes should include:
- A failing test (before fix)
- Passing test (after fix)

### ✔ Performance-aware

Qlue-ls may process large or adversarial inputs.

Avoid:
- Unbounded allocations
- Excessive cloning
- Accidental quadratic behavior

If your change impacts performance, mention it.

---

## Coding Guidelines

- Follow idiomatic Rust.
- Prefer clarity over cleverness.
- Avoid unnecessary dependencies.
- Keep public APIs minimal.
- Document non-obvious decisions.

If something feels overly complex, it probably is.

---

## Language Server Considerations

Qlue-ls:

- Runs locally on user machines
- May process untrusted source files
- Must be robust against malformed input

Changes must not:

- Introduce filesystem access beyond intended scope
- Execute external commands without clear justification
- Break WASM sandbox expectations

Security-sensitive changes may receive extra scrutiny.

---

## Commit Messages

Use clear, descriptive commit messages.

Good:

```
FIX(parser): prevent infinite recursion on malformed input
```

Bad:

```
fix stuff
```

---

## Pull Request Checklist

Before opening a PR, confirm:

- [ ] The project builds (see README)
- [ ] Tests pass
- [ ] Code is formatted (`cargo fmt`)
- [ ] Lints pass (`cargo clippy`)
- [ ] Changes are explained clearly
- [ ] Scope is limited to one concern

---

## What Not to Submit

- Drive-by refactors without clear benefit
- Large architectural rewrites without discussion
- Style-only changes across unrelated files
- New dependencies without strong justification

---

## Code of Conduct

By participating, you agree to follow the project's Code of Conduct.

Be respectful. Be constructive.

---

## Final Thoughts

Qlue-ls aims to be:

- Predictable
- Secure
- Performant
- Maintainable

Contributions that move the project in that direction are welcome.

If you’re unsure about anything, open a discussion first.
