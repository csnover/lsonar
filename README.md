# What is it?
This is the Lua 5.3 pattern engine, fully compatible with the original Lua 5.3 engine.

# Contribution
If you find any bug, please create an `Issue`. If you have already solved it, create a `Pull Request` and I will address it at the earliest opportunity.

# TODO
- Translate [`Error::Lexer`], [`Error::Parser`], [`Error::Matcher`] to readable enum variants instead of string

# То, что нужно сделать
- Полностью удалить все текущие тесты, переписать их в отдельную папку tests, и переписать тесты `string.find` из `strings.lua` (оригинальные тесты), особое внимание обратить на то, как парсер реагирует на UTF-8, Unicode, как работает с жадными/нежадными quantifiers, и как работает синтаксис паттернов (как обрабатываются ошибки в случае когда НЕЛЬЗЯ, не выкидывает ли ошибки в случаях когда МОЖНО, выдаёт ли верные ответы). Проверить абсолютно ВСЕ сценарии и синтаксис;