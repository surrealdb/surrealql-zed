; Outline / document symbols for schema statements.
; Node names mirror @surrealdb/lezer (PascalCase). DefineStatement is flat:
; the leading DEFINE keyword, the entity keyword, then the name node, whose
; type varies by entity (Ident / Idiom / FunctionName / VariableName).

; DEFINE NAMESPACE | DATABASE | TABLE | EVENT | INDEX | ANALYZER | USER | TOKEN
; DEFINE FIELD (Idiom) | FUNCTION (FunctionName) | PARAM (VariableName)
(DefineStatement
  . (Keyword) @context
  . (Keyword) @context
  . [
      (Ident)
      (Idiom)
      (FunctionName)
      (VariableName)
    ] @name) @item

; DEFINE ACCESS <name> ...
(DefineStatement
  . (Keyword) @context
  . (AccessDefinition
      . (Keyword) @context
      . (Ident) @name)) @item

; DEFINE SCOPE <name> ...
(DefineStatement
  . (Keyword) @context
  . (ScopeDefinition
      . (Keyword) @context
      . (Ident) @name)) @item

; ALTER TABLE <name> ...
(AlterStatement
  . (Keyword) @context
  . (Keyword) @context
  . [
      (Ident)
      (FunctionName)
    ] @name) @item

; REMOVE TABLE | FIELD | FUNCTION | ... <name>
(RemoveStatement
  . (Keyword) @context
  . (Keyword) @context
  . [
      (Ident)
      (Idiom)
      (FunctionName)
      (VariableName)
    ] @name) @item
