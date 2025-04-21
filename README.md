# What is it?
This is the Lua 5.3 pattern engine, fully compatible with the original Lua 5.3 engine.

# Contribution
If you find any bug, please create an `Issue`. If you have already solved it, create a `Pull Request` and I will address it at the earliest opportunity.

# TODO
- Implement [`CaptureRef`] to allow full implementation of [`string.gsub`]
- Implement some specific things to allow full implementation of [`string.match`] and [`string.gmatch`]
- Translate [`Error::Lexer`], [`Error::Parser`], [`Error::Matcher`] to readable enum variants instead of string