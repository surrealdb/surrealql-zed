; SurrealQL highlight queries — node names mirror @surrealdb/lezer (PascalCase).
; Mapping follows packages/lezer-surrealql/src/highlight.js

; Keywords and literals
(Keyword) @keyword
(Bool) @boolean
(None) @constant
(Literal) @constant.builtin

; Identifiers and names
(Ident) @variable
(VariableName) @variable
(KeyName) @property
(ObjectKey) @property
(TypeName) @type
(FunctionName) @function
(FunctionCall) @function

; Record identifiers
(RecordTbIdent) @type
(RecordIdIdent) @type
(RecordIdString) @type

; Strings, numbers, regex
(String) @string
(FormatString) @string.special
(Regex) @string.regex
(Int) @number
(Float) @number.float
(Decimal) @number
(VersionNumber) @number
(Duration) @number
(DurationPart) @number

; Comments
(Comment) @comment
(BlockComment) @comment.block

; Operators / punctuation
(PrefixExpression) @operator
(Operator) @operator
(RangeOp) @operator
(LookupRight) @operator
(LookupLeft) @operator
(LookupBoth) @operator
(Any) @operator
(At) @operator
(Optional) @operator
(Pipe) @punctuation.special
(Colon) @punctuation.delimiter
(BraceOpen) @punctuation.bracket
(BraceClose) @punctuation.bracket
"[" @punctuation.bracket
"]" @punctuation.bracket
"<" @punctuation.bracket
">" @punctuation.bracket
"," @punctuation.delimiter
"|" @punctuation.delimiter

; Clause / token keywords aliased through literal nodes
(Distance) @constant.builtin
(Filter) @constant.builtin
(AnalyzerTokenizer) @constant.builtin
(TokenType) @constant.builtin
(HttpMethod) @constant.builtin
(IndexTypeClause) @type

; Embedded JavaScript in scripting functions
(FunctionJs) @keyword.function
(JavaScriptBlock) @embedded
