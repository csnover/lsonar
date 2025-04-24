# What is it?
This is the Lua 5.3 pattern engine, fully compatible with the original Lua 5.3 engine.

# Contribution
If you find any bug, please create an `Issue`. If you have already solved it, create a `Pull Request` and I will address it at the earliest opportunity.

# TODO
- **Refactor:** Errors should be enum variants, not [`String`]

# Known Issues
- In the test [`test_find_pattern_with_captures`], we are only capturing two `%d`s in the first capture (`20`), although we should capture the whole string.