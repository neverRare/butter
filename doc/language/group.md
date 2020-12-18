# Group

The group for expressions, often operations or chain of operations. These can override default [precedence].

[precedence]: ./miscellaneous.md#operator-precedence

```butter
-- The following are similar, only the latter is more explicit
-- These evaluates to 7
1 + 3 * 2;
1 + (3 * 2);

-- The following is different from above expressions. This is 8
(1 + 3) * 2;
```
