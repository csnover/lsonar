# What is it?
This is the Lua 5.3 pattern engine, fully compatible with the original Lua 5.3 engine.

# Contribution
If you find any bug, please create an `Issue`. If you have already solved it, create a `Pull Request` and I will address it at the earliest opportunity.

# TODO
- **Fix:** Errors should be enum variants, not [`String`]
- **Refactor:** Completely remove all current tests, rewrite them in a separate `tests` folder, and rewrite the [`string.find`] tests from `strings.lua` (the original tests), paying special attention to how the parser reacts to UTF-8, Unicode, how it works with greedy/non-greedy quantifiers, and how pattern syntax works (how errors are handled in cases where it's NOT allowed, whether it throws errors in cases where it IS allowed, whether it returns correct results). Check absolutely ALL scenarios and syntax;